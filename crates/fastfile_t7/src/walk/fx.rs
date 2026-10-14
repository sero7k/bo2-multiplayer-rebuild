//! Effects (`FxEffectDef`, 0x90; `FxElemDef`, 0x260). No public T7 layout:
//! read on retail `mp_common` (its 515 effects), with the Black Ops II
//! `FxElemDef` as the guide to what the fields are.
//!
//! ```text
//! FxEffectDef 0x90: 0x00 char* name; 0x08 u16 flags; 0x0a u8 efPriority;
//!             0x0c i16 elemDefCountLooping, 0x0e elemDefCountOneShot, 0x10 elemDefCountEmission;
//!             0x14 i32 totalSize (the effect's stream bytes, header included, on most effects);
//!             0x18 i32 msecLoopingLife, 0x1c msecNonLoopingLife; 0x20 FxElemDef* elemDefs;
//!             0x58 char* (a sound alias name, loaded after the elements)
//! FxElemDef  0x260: 0xc8 u8 elemType, visualCount, velIntervalCount (a), visStateIntervalCount (b),
//!             0xcc u8 c, 0xcd u8 d;
//!             0xd0 velSamples ((a + 1) x 96); 0xd8 (b + 1) x 8; 0xe0 visSamples ((d + 1) x 80);
//!             0xe8 (c + 1) x 8;
//!             0xf0 FxElemDefVisuals (0x40: one inline FxElemVisuals, or an array pointer);
//!             0x130 a second FxElemDefVisuals (images beside the materials);
//!             0x198, 0x1a0, 0x1a8, 0x1d0 char* (effect names: impact, death, emitted, attached);
//!             0x1e8 extended (trail def for trails, a light description or lens flare);
//!             0x208, 0x218 char* (sound names)
//! FxElemVisuals 0x40: 0x00 the asset pointer (material, model, light description, or a name)
//! FxElemMarkVisuals 24: three Material*
//! FxTrailDef 0x30: 0x14 i32 vertCount; 0x18 verts (20 each); 0x20 i32 indCount; 0x28 u16* inds
//! ```
//!
//! Element types (`elemType`), by what their visuals name: 0..=6 sprites,
//! tails, lines, trails and clouds (materials; 5 is the trail), 7 models,
//! 8 lights (light descriptions), 12 lens flares (the `klf` in `extended`,
//! no visuals), 13 decals (mark visuals), 14 sounds or runners (names).
//!
//! Stream order: name, the element array, then per element: the four
//! sample arrays in field order, the visuals (an array of `visualCount`
//! 0x40 visuals when `visualCount > 1`, else the inline one's pointer),
//! the second visuals the same way, the four effect names, the extended
//! def, the two sound names; after all elements the header's sound name.
//! The sizes add up: on 316 of the 334 effects whose name follows them, the
//! next asset starts where the walk ends and `totalSize` matches (on the
//! rest `totalSize` is a few bytes off while the next asset still starts
//! where the walk ends).

use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};
use crate::stream::Stream;

const ELEM: usize = 0x260;
const VISUALS: usize = 0x40;
const MARK_VISUALS: usize = 24;
const TRAIL: usize = 0x30;
const TRAIL_VERTEX: usize = 20;

/// The sample arrays: (field, record size, count byte).
const SAMPLES: [(usize, usize, usize); 4] = [(0xd0, 96, 0xca), (0xd8, 8, 0xcb), (0xe0, 80, 0xcd), (0xe8, 8, 0xcc)];
const NAMES: [usize; 4] = [0x198, 0x1a0, 0x1a8, 0x1d0];
const SOUNDS: [usize; 2] = [0x208, 0x218];
const EXTENDED: usize = 0x1e8;
const POINTERS: [usize; 13] = [0xd0, 0xd8, 0xe0, 0xe8, 0xf0, 0x130, 0x198, 0x1a0, 0x1a8, 0x1d0, 0x1e8, 0x208, 0x218];

const TYPE_TRAIL: u8 = 5;
const TYPE_MODEL: u8 = 7;
const TYPE_LIGHT: u8 = 8;
const TYPE_LENS_FLARE: u8 = 12;
const TYPE_DECAL: u8 = 13;
const TYPE_NAMED: u8 = 14;

/// The asset or string one visual's pointer at `field` names.
fn visual(s: &mut Stream, field: usize, ty: u8, second: bool) -> Result<()> {
    match ty {
        0..=6 if second => s.asset(AssetType::Image, field).map(drop),
        0..=6 => s.asset(AssetType::Material, field).map(drop),
        TYPE_MODEL => s.asset(AssetType::XModel, field).map(drop),
        TYPE_LIGHT => s.asset(AssetType::LightDescription, field).map(drop),
        TYPE_NAMED => s.string(field).map(drop),
        // A lens flare element's visuals hold hashes, not a pointer.
        TYPE_LENS_FLARE => Ok(()),
        _ if !s.ptr_at(field)?.is_inline() => s.asset(AssetType::Material, field).map(drop),
        _ => Err(ReadError::Invalid("effect visual of an unknown element type", field)),
    }
}

/// Whether the asset a pointer at `field` loads right here is a material
/// rather than an image: an image's header holds its name pointer at 0xf8
/// (`-1`, the name following) and none at 0; a material's the reverse.
fn material_follows(s: &Stream, field: usize) -> Result<bool> {
    if !s.ptr_at(field)?.is_inline() {
        return Ok(false);
    }
    let at = s.pos();
    let image_name = super::image::IMAGE_NAME_AT;
    Ok(s.u64_at(at)? == u64::MAX && s.u64_at(at + image_name).map_or(true, |w| w != u64::MAX))
}

/// An `FxElemDefVisuals` at `field` (0x40).
fn visuals(s: &mut Stream, field: usize, ty: u8, count: usize, second: bool) -> Result<()> {
    if ty == TYPE_DECAL {
        if let Some(a) = s.records(field, count, MARK_VISUALS, 7)? {
            for i in 0..count * 3 {
                // (The second visuals are images, but where an image would
                // load a material does on `zm_genesis`'s
                // `vdd/gfx_decal_liquid_pool` effect: a header with a
                // name first and no image name at 0xf8.)
                let t = if second && !material_follows(s, a + 8 * i)? { AssetType::Image } else { AssetType::Material };
                s.asset(t, a + 8 * i)?;
            }
        }
    } else if count > 1 {
        if let Some(a) = s.records(field, count, VISUALS, 7)? {
            for i in 0..count {
                s.no_loads(a + VISUALS * i, VISUALS, &[0])?;
                visual(s, a + VISUALS * i, ty, second)?;
            }
        }
    } else {
        visual(s, field, ty, second)?;
    }
    Ok(())
}

fn extended(s: &mut Stream, field: usize, ty: u8) -> Result<()> {
    match ty {
        TYPE_TRAIL => {
            if let Some(t) = s.array(field, TRAIL, 7)? {
                s.no_loads(t, TRAIL, &[0x18, 0x28])?;
                let verts = s.i32_at(t + 0x14)?.max(0) as usize;
                s.records(t + 0x18, verts, TRAIL_VERTEX, 3)?;
                let inds = s.i32_at(t + 0x20)?.max(0) as usize;
                s.records(t + 0x28, inds, 2, 1)?;
            }
            Ok(())
        }
        TYPE_LIGHT => s.asset(AssetType::LightDescription, field).map(drop),
        TYPE_LENS_FLARE => s.asset(AssetType::NewLensFlareDef, field).map(drop),
        _ if !s.ptr_at(field)?.is_inline() => s.array(field, 0, 0).map(drop),
        _ => Err(ReadError::Invalid("effect extended def of an unknown element type", field)),
    }
}

fn elem(s: &mut Stream, e: usize) -> Result<()> {
    s.no_loads(e, ELEM, &POINTERS)?;
    let ty = s.u8_at(e + 0xc8)?;
    let count = s.u8_at(e + 0xc9)? as usize;
    for (field, size, count_at) in SAMPLES {
        let n = s.u8_at(e + count_at)? as usize + 1;
        s.records(e + field, n, size, 3)?;
    }
    visuals(s, e + 0xf0, ty, count, false)?;
    visuals(s, e + 0x130, ty, count, true)?;
    for f in NAMES {
        s.string(e + f)?;
    }
    extended(s, e + EXTENDED, ty)?;
    for f in SOUNDS {
        s.string(e + f)?;
    }
    Ok(())
}

pub fn fx(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::Fx.header_size(), &[0, 0x20, 0x58])?;
    let name = s.string(h)?;
    let n = (0..3).map(|i| s.u16_at(h + 0xc + 2 * i).map(|v| (v as i16).max(0) as usize)).sum::<Result<usize>>()?;
    if let Some(t) = s.records(h + 0x20, n, ELEM, 7)? {
        for i in 0..n {
            elem(s, t + ELEM * i)?;
        }
    }
    s.string(h + 0x58)?;
    Ok(name)
}

/// `NewLensFlareDef` (`klf`, 0xa8), read on retail `mp_common` (no public
/// layout):
///
/// ```text
/// 0x00 char* name; 0x08 LensFlareElement* elements; 0x10 u8* curves (PHYSICAL);
/// 0x18 u8* (the first buffer's data again); 0x20 LensFlareBuffer;
/// 0x50 u8*; 0x58 LensFlareBuffer (the curves); 0x90 u16 elementCount, bufferCount;
/// 0x98 GfxImage* (a `-2` pointer); 0xa0 u32
/// LensFlareBuffer 0x30: 0x00 u8* data (PHYSICAL); 0x08 u32 size; 0x0c u32 count
/// LensFlareElement 0x1f0: 0x1a8 LensFlareBuffer; 0x1e0 GfxImage* image
/// ```
///
/// Stream order: the name, the elements (VIRTUAL, aligned 16), per element
/// its buffer's data (`size * count` bytes, PHYSICAL, aligned 256) and its
/// image (an asset; later elements name an earlier one's by reference),
/// then the curves (the second buffer's `size * count` bytes, PHYSICAL,
/// aligned 16), then the image at 0x98. The header's buffers name the
/// asset's own PHYSICAL data (the first element's buffer and the curves)
/// ahead of loading it: forward references, checked at the end of the walk.
pub fn lens_flare(s: &mut Stream, h: usize) -> Result<Option<String>> {
    const ELEMENT: usize = 0x1f0;
    s.no_loads(
        h,
        AssetType::NewLensFlareDef.header_size(),
        &[0, 8, 0x10, 0x98],
    )?;
    let name = s.string(h)?;
    let buffer = |s: &mut Stream, b: usize, mask: u64| -> Result<()> {
        let size = s.u32_at(b + 8)? as usize * s.u32_at(b + 0xc)? as usize;
        s.push(crate::stream::PHYSICAL);
        let r = s.array(b, size, mask);
        s.pop();
        r.map(drop)
    };
    for f in [0x18, 0x50] {
        s.push(crate::stream::PHYSICAL);
        let r = s.array(h + f, 0, 0);
        s.pop();
        r?;
    }
    buffer(s, h + 0x20, 255)?;
    let n = s.u16_at(h + 0x90)? as usize;
    if let Some(t) = s.records(h + 8, n, ELEMENT, 15)? {
        for i in 0..n {
            let e = t + ELEMENT * i;
            s.no_loads(e, ELEMENT, &[0x1a0, 0x1a8, 0x1e0])?;
            buffer(s, e + 0x1a8, 255)?;
            s.asset(AssetType::Image, e + 0x1e0)?;
        }
    }
    let curves = s.u32_at(h + 0x60)? as usize * s.u32_at(h + 0x64)? as usize;
    s.push(crate::stream::PHYSICAL);
    let r = s.array(h + 0x10, curves, 15);
    s.pop();
    r?;
    s.array(h + 0x58, 0, 0)?;
    s.asset(AssetType::Image, h + 0x98)?;
    Ok(name)
}
