//! bo2mp: Black Ops II's final colour, from the map's own vision file
//! (`vision/<map>.vision`) and colour table (`GfxWorld::lutMaterial`).
//!
//! The game builds a 32x32x32 colour table each frame with
//! `hdr_create_lut2dv` (fxc disassembly of its pixel shader) and looks the
//! frame up in it after bloom and the highlight roll-off (`hdr_bloom_apply`):
//!
//! - start from the map's own table (band 0 of the lut material's image,
//!   gamma values; the band picked when no LUT volume holds the eye),
//! - weights by its luminance (Rec. 709, on the gamma values): shadows a
//!   falling ramp `vc_RS.x..vc_RE.x`, highlights a rising ramp `y`,
//!   midtones a rising ramp `z` times a falling ramp `w`; each smoothstep,
//!   the three normalised,
//! - in linear (squared): the shadow, midtone and highlight matrices
//!   (`vc_SM*`, `vc_MM*`, `vc_HM*`, rows with an offset) mixed by those
//!   weights, clamped; then per channel the power `vc_FGM`, the saturation
//!   `vc_FSM` (weights, amount), the blend back toward the input `vc_FBM`,
//!   then the code's colour matrix (identity: the vision's levels are),
//!   clamped and square-rooted.
//!
//! The vision's bloom rows (`vc_RGBH`, `vc_RGBL`, `vc_YH`, `vc_YL`) are the
//! tints `hdr_bloom_combine_hilo` puts the high and low bloom levels'
//! colour and luminance back with.

use std::sync::{Arc, RwLock};

/// The `vc_*` rows of a Black Ops II vision file (four floats each).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct T6Vision {
    pub film_enable: bool,
    pub levels_in_black: [f32; 4],
    pub levels_in_white: [f32; 4],
    pub levels_in_gamma: [f32; 4],
    pub levels_out_black: [f32; 4],
    pub levels_out_white: [f32; 4],
    pub bloom_rgb_hi: [f32; 4],
    pub bloom_rgb_lo: [f32; 4],
    pub bloom_y_hi: [f32; 4],
    pub bloom_y_lo: [f32; 4],
    pub range_start: [f32; 4],
    pub range_end: [f32; 4],
    pub shadow: [[f32; 4]; 3],
    pub highlight: [[f32; 4]; 3],
    pub midtone: [[f32; 4]; 3],
    pub final_gamma: [f32; 4],
    pub final_saturation: [f32; 4],
    pub final_blend: [f32; 4],
}

impl Default for T6Vision {
    /// What changes nothing: identity levels and matrices, full saturation.
    fn default() -> Self {
        let row = |i: usize| {
            let mut r = [0.0; 4];
            r[i] = 1.0;
            r
        };
        let identity = [row(0), row(1), row(2)];
        Self {
            film_enable: false,
            levels_in_black: [0.0; 4],
            levels_in_white: [32.0; 4],
            levels_in_gamma: [1.0; 4],
            levels_out_black: [0.0; 4],
            levels_out_white: [32.0; 4],
            bloom_rgb_hi: [0.0; 4],
            bloom_rgb_lo: [0.0; 4],
            bloom_y_hi: [0.0; 4],
            bloom_y_lo: [0.0; 4],
            range_start: [0.0, 0.0, 0.0, 0.5],
            range_end: [1.0, 1.0, 0.5, 1.0],
            shadow: identity,
            highlight: identity,
            midtone: identity,
            final_gamma: [1.0, 1.0, 1.0, 0.0],
            final_saturation: [T6_LUMA[0], T6_LUMA[1], T6_LUMA[2], 1.0],
            final_blend: [1.0, 1.0, 1.0, 0.0],
        }
    }
}

/// The luminance weights BO2's post shaders use.
pub const T6_LUMA: [f32; 3] = [0.212585, 0.715195, 0.07222];

/// Parse a vision file's text: `name "values"` per line. Rows it does not
/// name keep their identity value; `None` when it names no `vc_` row.
pub fn parse_t6_vision(text: &str) -> Option<T6Vision> {
    let mut v = T6Vision::default();
    let mut found = false;
    for line in text.lines() {
        let line = line.trim();
        let Some((name, rest)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let value = rest.trim().trim_matches('"');
        let mut four = [0.0f32; 4];
        let mut n = 0;
        for (slot, part) in four.iter_mut().zip(value.split_whitespace()) {
            let Ok(x) = part.parse::<f32>() else {
                break;
            };
            *slot = x;
            n += 1;
        }
        let name = name.to_ascii_lowercase();
        if name == "r_filmenable" {
            v.film_enable = value.trim() != "0";
            continue;
        }
        if n != 4 {
            continue;
        }
        let target = match name.as_str() {
            "vc_lib" => &mut v.levels_in_black,
            "vc_liw" => &mut v.levels_in_white,
            "vc_lig" => &mut v.levels_in_gamma,
            "vc_lob" => &mut v.levels_out_black,
            "vc_low" => &mut v.levels_out_white,
            "vc_rgbh" => &mut v.bloom_rgb_hi,
            "vc_rgbl" => &mut v.bloom_rgb_lo,
            "vc_yh" => &mut v.bloom_y_hi,
            "vc_yl" => &mut v.bloom_y_lo,
            "vc_rs" => &mut v.range_start,
            "vc_re" => &mut v.range_end,
            "vc_smr" => &mut v.shadow[0],
            "vc_smg" => &mut v.shadow[1],
            "vc_smb" => &mut v.shadow[2],
            "vc_hmr" => &mut v.highlight[0],
            "vc_hmg" => &mut v.highlight[1],
            "vc_hmb" => &mut v.highlight[2],
            "vc_mmr" => &mut v.midtone[0],
            "vc_mmg" => &mut v.midtone[1],
            "vc_mmb" => &mut v.midtone[2],
            "vc_fgm" => &mut v.final_gamma,
            "vc_fsm" => &mut v.final_saturation,
            "vc_fbm" => &mut v.final_blend,
            _ => continue,
        };
        *target = four;
        found = true;
    }
    found.then_some(v)
}

/// Side of the colour table.
pub const T6_LUT_SIZE: usize = 32;

/// One colour (gamma values, the table's input) through `hdr_create_lut2dv`
/// with `alpha` the base table's alpha. Returns gamma values.
pub fn t6_grade_colour(v: &T6Vision, c: [f32; 3], alpha: f32) -> [f32; 3] {
    let dot3 = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let l = dot3(c, T6_LUMA);
    // The code passes each ramp as scale and bias (mad_sat): rising from
    // start to end, or falling.
    let ramp = |i: usize, rising: bool| {
        let (s, e) = (v.range_start[i], v.range_end[i]);
        let t = if (e - s).abs() < 1e-6 {
            if l >= s { 1.0 } else { 0.0 }
        } else {
            ((l - s) / (e - s)).clamp(0.0, 1.0)
        };
        if rising { t } else { 1.0 - t }
    };
    let smooth = |x: f32| x * x * (3.0 - 2.0 * x);
    let mut w = [
        smooth(ramp(0, false)),
        smooth(ramp(2, true) * ramp(3, false)),
        smooth(ramp(1, true)),
    ];
    let sum = w[0] + w[1] + w[2];
    if sum > 0.0 {
        w = w.map(|x| x / sum);
    }
    let lin = c.map(|x| x * x);
    let row = |r: [f32; 4]| r[0] * lin[0] + r[1] * lin[1] + r[2] * lin[2] + r[3] * alpha;
    let mut out = [0.0f32; 3];
    for (ch, o) in out.iter_mut().enumerate() {
        let graded =
            w[0] * row(v.shadow[ch]) + w[1] * row(v.midtone[ch]) + w[2] * row(v.highlight[ch]);
        *o = graded.clamp(0.0, 1.0).powf(v.final_gamma[ch]);
    }
    let fs = v.final_saturation;
    let lum = dot3(out, [fs[0], fs[1], fs[2]]);
    for (ch, o) in out.iter_mut().enumerate() {
        let saturated = lum + fs[3] * (*o - lum);
        let blended = lin[ch] + v.final_blend[ch] * (saturated - lin[ch]);
        *o = blended.clamp(0.0, 1.0).sqrt();
    }
    out
}

/// The 32x32x32 table, RGBA8, red fastest then green then blue (a 3D
/// texture's layout). `base` is the map's own table: band 0 of its lut
/// image (1024x32 RGBA8, texel (b * 32 + r, g)); `None` = identity.
pub fn t6_grade_lut(v: &T6Vision, base: Option<&[u8]>) -> Vec<u8> {
    let n = T6_LUT_SIZE;
    let base = base.filter(|b| b.len() >= n * n * n * 4);
    let mut out = Vec::with_capacity(n * n * n * 4);
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                let (c, a) = match base {
                    Some(px) => {
                        let o = (g * n * n + b * n + r) * 4;
                        (
                            [0, 1, 2].map(|i| f32::from(px[o + i]) / 255.0),
                            f32::from(px[o + 3]) / 255.0,
                        )
                    }
                    None => ([r, g, b].map(|x| x as f32 / (n - 1) as f32), 1.0),
                };
                let graded = t6_grade_colour(v, c, a);
                for x in graded {
                    out.push((x * 255.0).round().clamp(0.0, 255.0) as u8);
                }
                out.push(255);
            }
        }
    }
    out
}

/// What the renderer's final pass draws a Black Ops II map with.
#[derive(Clone, Debug)]
pub struct T6Grade {
    /// `t6_grade_lut`'s table.
    pub lut: Vec<u8>,
    pub vision: T6Vision,
}

fn grade_slot() -> &'static RwLock<(u64, Option<Arc<T6Grade>>)> {
    static SLOT: std::sync::OnceLock<RwLock<(u64, Option<Arc<T6Grade>>)>> =
        std::sync::OnceLock::new();
    SLOT.get_or_init(Default::default)
}

/// Installed once per map load (`None`: the map has no vision of its own,
/// as every other game's map: drawn as its colours are).
pub fn install_t6_grade(grade: Option<T6Grade>) {
    let grade = grade.map(Arc::new);
    if let Ok(mut m) = map_slot().write() {
        *m = grade.clone();
    }
    if let Ok(mut n) = naked_slot().write() {
        *n = (grade.clone(), None);
    }
    put_grade(grade);
}

/// bo2mp: the loaded map's own grade (what [`install_t6_grade`] put), which
/// a script's `visionsetnaked` of a vision no zone holds goes back to.
fn map_slot() -> &'static RwLock<Option<Arc<T6Grade>>> {
    static SLOT: std::sync::OnceLock<RwLock<Option<Arc<T6Grade>>>> = std::sync::OnceLock::new();
    SLOT.get_or_init(Default::default)
}

/// bo2mp: a map from another game under Black Ops II's rules: no vision and
/// no colour table of its own, so its grade is none (its colours as they
/// are), and nothing of the Black Ops II map before it stays.
pub fn install_foreign_map_grade() {
    set_t6_vision_base(None);
    install_t6_grade(None);
}

/// bo2mp: one colour (gamma values) through an Infinity Ward / Treyarch
/// film vision (`r_film*`, CoD4 to MW3), as their film shaders draw it.
/// CoD4's `vertcol_film` (disassembled from its code_post_gfx_mp zone) and
/// the MW2 constants the native path packs (`postfx_plan::film_sources`):
/// luminance L (0.299, 0.587, 0.114); desaturation by L from its dark
/// value to its full one; times the contrast and the tint, a quadratic
/// Bezier from the dark tint through the medium to the light by L (CoD4
/// has no medium: the midpoint, a straight line); plus brightness + 0.5 -
/// 0.5 * contrast; inverted when asked. Clamped to 0..1.
pub fn iw_film_colour(v: &crate::FilmVision, c: [f32; 3]) -> [f32; 3] {
    let l = c[0] * 0.299 + c[1] * 0.587 + c[2] * 0.114;
    let s = v.desaturation_dark + (v.desaturation - v.desaturation_dark) * l;
    let (mut contrast, mut offset) = (v.contrast, v.brightness + 0.5 - 0.5 * v.contrast);
    if v.invert {
        contrast = -contrast;
        offset += 1.0;
    }
    std::array::from_fn(|i| {
        let grey = c[i] + (l - c[i]) * s;
        let tint = v.dark_tint[i]
            + 2.0 * l * (v.medium_tint[i] - v.dark_tint[i])
            + l * l * (v.light_tint[i] - 2.0 * v.medium_tint[i] + v.dark_tint[i]);
        (grey * contrast * tint + offset).clamp(0.0, 1.0)
    })
}

/// bo2mp: a film vision as the frame's colour table (the layout of
/// [`t6_grade_lut`]): the identity table through [`iw_film_colour`].
pub fn iw_film_lut(v: &crate::FilmVision) -> Vec<u8> {
    let n = T6_LUT_SIZE;
    let mut out = Vec::with_capacity(n * n * n * 4);
    for b in 0..n {
        for g in 0..n {
            for r in 0..n {
                let c = [r, g, b].map(|x| x as f32 / (n - 1) as f32);
                for x in iw_film_colour(v, c) {
                    out.push((x * 255.0).round() as u8);
                }
                out.push(255);
            }
        }
    }
    out
}

/// bo2mp: a map from another game under Black Ops II's rules, with its own
/// game's film vision (`vision/<map>.vision`): that vision is its grade
/// (no bloom: those games' visions tint no bloom levels). Without one, or
/// with the film off, as [`install_foreign_map_grade`].
pub fn install_film_map_grade(vision: Option<&crate::FilmVision>) -> bool {
    match vision.filter(|v| v.enable) {
        Some(v) => {
            set_t6_vision_base(None);
            install_t6_grade(Some(T6Grade { lut: iw_film_lut(v), vision: T6Vision::default() }));
            true
        }
        None => {
            install_foreign_map_grade();
            false
        }
    }
}

fn put_grade(grade: Option<Arc<T6Grade>>) {
    if let Ok(mut s) = grade_slot().write() {
        s.0 += 1;
        s.1 = grade;
    }
}

/// bo2mp vehicle screens: the naked vision's grade (the map's own, or a
/// script's `visionsetnaked`) and the server's vision for the local player
/// over it (`setvisionsetforplayer` while `useservervisionset` is on).
type NakedSlot = RwLock<(Option<Arc<T6Grade>>, Option<String>)>;
fn naked_slot() -> &'static NakedSlot {
    static SLOT: std::sync::OnceLock<NakedSlot> = std::sync::OnceLock::new();
    SLOT.get_or_init(Default::default)
}

fn named_grade(name: &str) -> Option<Arc<T6Grade>> {
    let found = vision_registry()
        .read()
        .ok()
        .and_then(|r| r.0.get(&name.to_ascii_lowercase()).map(|v| (*v, r.1.clone())));
    match found {
        Some((vision, base)) => Some(Arc::new(T6Grade { lut: t6_grade_lut(&vision, base.as_deref()), vision })),
        // An IW-style film vision of that name (`register_film_visions`).
        None => film_registry()
            .read()
            .ok()
            .and_then(|r| r.get(&name.to_ascii_lowercase()).cloned())
            .map(|v| Arc::new(T6Grade { lut: iw_film_lut(&v), vision: T6Vision::default() })),
    }
}

fn film_registry() -> &'static RwLock<std::collections::HashMap<String, crate::FilmVision>> {
    static REG: std::sync::OnceLock<RwLock<std::collections::HashMap<String, crate::FilmVision>>> =
        std::sync::OnceLock::new();
    REG.get_or_init(Default::default)
}

/// bo2mp: named visions in the IW games' film form (contrast, brightness,
/// desaturation, light/dark tints), drawn as Black Ops II's colour table
/// when a script's `visionsetnaked` names them (Black Ops II's own of a
/// name win).
pub fn register_film_visions(visions: Vec<(String, crate::FilmVision)>) {
    if let Ok(mut r) = film_registry().write() {
        for (name, v) in visions {
            r.insert(name.to_ascii_lowercase(), v);
        }
    }
}

/// bo2mp vehicle screens: the local player's server vision ("" back to the
/// naked one). False when no zone holds the name (the naked one stays).
pub fn set_t6_player_vision(name: &str) -> bool {
    let name = name.trim();
    let over = if name.is_empty() { None } else { named_grade(name) };
    let found = name.is_empty() || over.is_some();
    let naked = naked_slot().write().ok().map(|mut n| {
        n.1 = over.is_some().then(|| name.to_ascii_lowercase());
        n.0.clone()
    });
    put_grade(over.or(naked.flatten()));
    found
}

/// bo2mp: every named vision the zones hold (`vision/<name>.vision`, lower
/// case) and the map's colour table band, so a script's `visionsetnaked`
/// (`mpOutro` at a match's end, back to the map's own after) can switch the
/// frame's grade.
fn vision_registry() -> &'static RwLock<(std::collections::HashMap<String, T6Vision>, Option<Vec<u8>>)> {
    static REG: std::sync::OnceLock<
        RwLock<(std::collections::HashMap<String, T6Vision>, Option<Vec<u8>>)>,
    > = std::sync::OnceLock::new();
    REG.get_or_init(Default::default)
}

/// Add named visions (a later one of a name wins).
pub fn register_t6_visions(visions: Vec<(String, T6Vision)>) {
    if let Ok(mut r) = vision_registry().write() {
        for (name, v) in visions {
            r.0.insert(name.to_ascii_lowercase(), v);
        }
    }
}

/// The installed map's colour table band 0 (what every vision's table is
/// built over), `None` when the map has none.
pub fn set_t6_vision_base(base: Option<Vec<u8>>) {
    if let Ok(mut r) = vision_registry().write() {
        r.1 = base;
    }
}

/// `visionsetnaked <name>`: install that vision's grade. False when no
/// zone holds the name: the map's own grade comes back. Black Ops II's
/// scripts name the map's own vision to end `mpIntro` (greyscale, before
/// the match) and `mpOutro`; a map from another game has none, and its
/// look must not stay grey for the whole match.
pub fn set_t6_vision(name: &str) -> bool {
    let found = named_grade(name);
    let is_found = found.is_some();
    let grade = found.or_else(|| map_slot().read().ok().and_then(|m| m.clone()));
    // (A server vision for the local player stays over it until cleared.)
    let over = naked_slot().write().ok().is_some_and(|mut n| {
        n.0 = grade.clone();
        n.1.is_some()
    });
    if !over {
        put_grade(grade);
    }
    is_found
}

/// The installed grade and its generation (bumped by every install).
pub fn t6_grade() -> (u64, Option<Arc<T6Grade>>) {
    grade_slot()
        .read()
        .map(|s| (s.0, s.1.clone()))
        .unwrap_or((0, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NUKETOWN_MP: &str = "r_filmEnable       \"1\"\n\
vc_LIB \"0.000000 0.000000 0.000000 0.000000\"\n\
vc_RGBH \"0.090000 0.090000 0.090000 0.650000\"\n\
vc_RS \"0.000000 0.000000 0.000000 0.500000\"\n\
vc_RE \"1.000000 1.000000 0.500000 1.000000\"\n\
vc_SMR \"1.164402 0.000000 0.000000 0.002557\"\n\
vc_FSM \"0.212585 0.715195 0.072220 0.880859\"\n";

    #[test]
    fn iw_film_reads_as_the_games_draw_it() {
        let off = crate::FilmVision { enable: true, ..Default::default() };
        // Contrast 1, no brightness, white tints, no desaturation: unchanged.
        for c in [[0.2, 0.5, 0.8], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]] {
            let out = iw_film_colour(&off, c);
            for i in 0..3 {
                assert!((out[i] - c[i]).abs() < 1e-5, "{c:?} -> {out:?}");
            }
        }
        // Full desaturation: grey at its luminance.
        let grey = crate::FilmVision { desaturation: 1.0, desaturation_dark: 1.0, ..off };
        let out = iw_film_colour(&grey, [1.0, 0.0, 0.0]);
        assert!((out[0] - 0.299).abs() < 1e-5 && (out[1] - 0.299).abs() < 1e-5);
        // Contrast about the middle, brightness on top.
        let punch = crate::FilmVision { contrast: 1.5, brightness: 0.1, ..off };
        let out = iw_film_colour(&punch, [0.5, 0.5, 0.5]);
        assert!((out[0] - 0.6).abs() < 1e-5);
        // The table is the right size and its corners follow the function.
        let lut = iw_film_lut(&punch);
        assert_eq!(lut.len(), T6_LUT_SIZE.pow(3) * 4);
        assert_eq!(lut[0], 0);
        assert_eq!(*lut.last().unwrap(), 255);
    }

    #[test]
    fn parses_rows() {
        let v = parse_t6_vision(NUKETOWN_MP).unwrap();
        assert!(v.film_enable);
        assert_eq!(v.bloom_rgb_hi, [0.09, 0.09, 0.09, 0.65]);
        assert_eq!(v.shadow[0], [1.164402, 0.0, 0.0, 0.002557]);
        assert_eq!(v.shadow[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(v.final_saturation[3], 0.880859);
        assert!(parse_t6_vision("r_filmEnable \"1\"\n").is_none());
    }

    #[test]
    fn default_vision_is_identity() {
        let v = T6Vision::default();
        for c in [
            [0.0, 0.0, 0.0],
            [0.25, 0.5, 0.75],
            [1.0, 1.0, 1.0],
            [0.9, 0.1, 0.3],
        ] {
            let o = t6_grade_colour(&v, c, 1.0);
            for i in 0..3 {
                assert!((o[i] - c[i]).abs() < 1e-5, "{c:?} -> {o:?}");
            }
        }
        let lut = t6_grade_lut(&v, None);
        assert_eq!(lut.len(), 32 * 32 * 32 * 4);
        // red fastest, then green, then blue
        assert_eq!(&lut[31 * 4..31 * 4 + 4], &[255, 0, 0, 255]);
        assert_eq!(&lut[32 * 31 * 4..32 * 31 * 4 + 4], &[0, 255, 0, 255]);
        assert_eq!(
            &lut[32 * 32 * 31 * 4..32 * 32 * 31 * 4 + 4],
            &[0, 0, 255, 255]
        );
    }

    /// bo2mp: `mpIntro` greys the frame before the match; the scripts then
    /// name the map's own vision. A map with none (another game's) must
    /// come back to its own grade, not stay grey.
    #[test]
    fn missing_vision_goes_back_to_the_maps_own() {
        let mut grey = T6Vision::default();
        grey.final_saturation[3] = 0.0;
        register_t6_visions(vec![("test_mpintro_grey".to_owned(), grey)]);
        install_foreign_map_grade();
        assert!(t6_grade().1.is_none());
        assert!(set_t6_vision("test_mpIntro_grey"));
        let during = t6_grade().1.expect("the intro's grade");
        assert_eq!(during.vision.final_saturation[3], 0.0);
        assert!(!set_t6_vision("test_mp_no_such_map"));
        assert!(t6_grade().1.is_none(), "the foreign map's own (no) grade is back");
        // A Black Ops II map's own grade comes back the same way.
        let own = T6Grade { lut: Vec::new(), vision: T6Vision::default() };
        install_t6_grade(Some(own));
        assert!(set_t6_vision("test_mpintro_grey"));
        assert!(!set_t6_vision("test_mp_no_such_map"));
        let back = t6_grade().1.expect("the map's own grade");
        assert_eq!(back.vision, T6Vision::default());
        install_foreign_map_grade();
    }

    #[test]
    fn saturation_pulls_toward_luminance() {
        let mut v = T6Vision::default();
        v.final_saturation[3] = 0.5;
        let o = t6_grade_colour(&v, [1.0, 0.0, 0.0], 1.0);
        let lum = T6_LUMA[0];
        assert!((o[0] - (lum + 0.5 * (1.0 - lum)).sqrt()).abs() < 1e-5);
        assert!((o[1] - (lum * 0.5).sqrt()).abs() < 1e-5);
    }
}
