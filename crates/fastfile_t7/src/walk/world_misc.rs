//! Asset types map zones load besides the world assets themselves (read
//! on retail map zones, `mp_nuketown_x` first; x64-zt has no T7 layout for
//! these): camera paths (`XCam`), ...
//!
//! Each walker reads its pointer fields in offset order, an array then
//! what its records point to, the way the other walkers do; layouts are
//! measured on the zones (the comments say where).

use crate::cursor::Result;
use crate::stream::Stream;

fn count(s: &Stream, at: usize) -> Result<usize> {
    Ok(s.u32_at(at)? as usize)
}

/// `XCam` (0xa8; `mp_nuketown_x`'s `ui_cam_endgame_mp_nuketown`):
///
/// ```text
/// 0x00 char* name; 0x08 char* (empty on the shipped cameras)
/// 0x18 u32 count A; 0x1c u32 frames; 0x20 u32 rate; 0x24 u32 count C
/// 0x48 A* (16: { u32; u32 n; frame28* })  frames: origin + quaternion
/// 0x50 B* (32: { u32; u32; u32 n; u32; frame48*; f32; u32 })
/// 0x60 C* (8 each)
/// 0x68 F* (16 each, count at 0x70: `zm_castle`'s and `zm_island`'s outro
///      cameras, two each; null on the MP cameras)
/// 0x78 D* (16 each, count at 0x80), 0x88 E* (16 each, count at 0x90)
/// ```
pub fn xcam(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::XCam.header_size(), &[0, 8, 0x48, 0x50, 0x60, 0x68, 0x78, 0x88])?;
    let name = s.string(h)?;
    s.string(h + 8)?;
    let n = count(s, h + 0x18)?;
    if let Some(t) = s.records(h + 0x48, n, 16, 7)? {
        for i in 0..n {
            let a = t + 16 * i;
            let frames = count(s, a + 4)?;
            s.records(a + 8, frames, 28, 3)?;
        }
    }
    // (The count is the byte at 0x14 on `mp_nuketown_x`'s cameras, at 0x15
    // on `zm_factory`'s intro camera.)
    // (`zm_stalingrad`'s `c_sta_outro_3rd_sh200_camera` has both bytes 0
    // and its count, one, in the u32 at 0x18.)
    let n = match (s.u8_at(h + 0x14)?, s.u8_at(h + 0x15)?) {
        (0, 0) => count(s, h + 0x18)?,
        (0, n) | (n, _) => n as usize,
    };
    if let Some(t) = s.records(h + 0x50, n, 32, 7)? {
        for i in 0..n {
            let b = t + 32 * i;
            let frames = count(s, b + 8)?;
            s.records(b + 0x10, frames, 48, 3)?;
        }
    }
    let n = count(s, h + 0x24)?;
    s.records(h + 0x60, n, 8, 3)?;
    // (`zm_castle`'s `c_cas_01_outro_3rd_sh010_cam`: two records of u32 ids
    // and counts between its frames and the list at 0x78.)
    let n = count(s, h + 0x70)?;
    s.records(h + 0x68, n, 16, 3)?;
    // Two lists of 16-byte records (u32 ids and counts; `zm_factory`'s
    // intro camera: five at 0x78, count at 0x80, three at 0x88, count at
    // 0x90).
    let n = count(s, h + 0x80)?;
    s.records(h + 0x78, n, 16, 3)?;
    let n = count(s, h + 0x90)?;
    s.records(h + 0x88, n, 16, 3)?;
    Ok(name)
}

/// `Glass` (136, `mp_nuketown_x`): 0x00 `GlassDef*`, 0x08 index, 0x0c brush
/// model, origin, angles, bounds, 0x40 planar, 0x41 outline vertex count,
/// 0x48 outline (8 each), axis, ...
pub const GLASS: usize = 136;
/// `GlassDef` (0x68): name, sizes, three materials (0x28..0x38), two
/// effect names (0x40, 0x48: strings), and references at 0x50..0x60.
pub const GLASS_DEF: usize = 0x68;

/// `Glasses` (0x58): `{ name; u32 numGlasses; Glass* glasses; u8*
/// workMemory; u32 workMemorySize; u8* smallAllocatorBlocks; u32
/// smallAllocatorSize; limits (u32s) }`. Order: name, the glasses (each:
/// its def, then its outline), the work memory, the small allocator
/// blocks (both filled with a byte pattern in the zone).
pub fn glasses(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::Glasses.header_size(), &[0, 0x10, 0x18, 0x28])?;
    let name = s.string(h)?;
    let n = count(s, h + 8)?;
    if let Some(t) = s.records(h + 0x10, n, GLASS, 7)? {
        for i in 0..n {
            let g = t + GLASS * i;
            s.no_loads(g, GLASS, &[0, 0x48])?;
            if let Some(d) = s.array(g, GLASS_DEF, 7)? {
                s.no_loads(d, GLASS_DEF, &[0, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60])?;
                s.string(d)?;
                for f in [0x28, 0x30, 0x38] {
                    s.asset(crate::AssetType::Material, d + f)?;
                }
                s.string(d + 0x40)?;
                s.string(d + 0x48)?;
                for f in [0x50, 0x58, 0x60] {
                    if s.ptr_at(d + f)?.is_inline() {
                        return Err(crate::cursor::ReadError::Invalid("glass def field (not read yet)", d + f));
                    }
                }
            }
            let verts = s.u8_at(g + 0x41)? as usize;
            s.records(g + 0x48, verts, 8, 3)?;
        }
    }
    // (The work memory is not VIRTUAL: the clip map's references after it
    // land only without it.)
    let k = count(s, h + 0x20)?;
    s.push(crate::stream::PHYSICAL);
    let r = s.array(h + 0x18, k, 15);
    s.pop();
    r?;
    let k = count(s, h + 0x30)?;
    s.array(h + 0x28, k, 15)?;
    Ok(name)
}

/// `SAnim` (siege animation, 0x70; `mp_freerun_02`'s
/// `p7_fxanim_gp_flag_freerun_01_sanim`): 0x00 name, 0x08
/// `StreamWrappedBuffer*` (its name a reference, its data resident or in
/// the `.xpak`), 0x18 records (28 each, count u16 at 0x64), bounds and
/// counts in between, 0x20 a second list (28 each: 16 bytes and three
/// u32s; count u16 at 0x6c: `mp_banzai`'s chains have two). Loaded in
/// that order.
pub fn sanim(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::SAnim.header_size(), &[0, 8, 0x18, 0x20])?;
    let name = s.string(h)?;
    super::gfxworld::stream_wrapped(s, h + 8)?;
    // (The first list's count: 3 on every MP siege animation read, where
    // the u16 at 0x66 is 1; `zm_zod`'s fungus pods carry 2 and 1, and their
    // list holds three. The list is the two counts' sum when the first is
    // under three; a 3 already counts the 1.)
    let (a, b) = (s.u16_at(h + 0x64)? as usize, s.u16_at(h + 0x66)? as usize);
    let n = if a < 3 { a + b } else { a };
    s.records(h + 0x18, n, 28, 3)?;
    let n = s.u16_at(h + 0x6c)? as usize;
    s.records(h + 0x20, n, 28, 3)?;
    Ok(name)
}

/// `StreamerHint` (0x28; read on `mp_sector`, `mp_crucible`): 0x00 name,
/// 0x08 u32 count, 0x10 hints (24 each: a name, a u32 kind, 8 bytes), 0x18
/// u32 count, 0x20 a second list (64 each). Order: the name, the hints,
/// each hint's name (pictures' and models' names, or references to earlier
/// ones), then the second list and its two arrays.
pub fn streamer_hint(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::StreamerHint.header_size(), &[0, 0x10, 0x20])?;
    let name = s.string(h)?;
    let n = count(s, h + 8)?;
    if let Some(t) = s.records(h + 0x10, n, 24, 7)? {
        for i in 0..n {
            s.string(t + 24 * i)?;
        }
    }
    // The second list (64 each; `mp_sector`, `mp_crucible`: one): 0x00 -1
    // (no pointer: nothing loads for it), 0x20 u32 count and 0x28 u16s,
    // 0x30 u32 count and 0x38 (u32, u32) pairs.
    let n = count(s, h + 0x18)?;
    if let Some(t) = s.records(h + 0x20, n, 64, 7)? {
        for i in 0..n {
            let r = t + 64 * i;
            let k = count(s, r + 0x20)?;
            s.records(r + 0x28, k, 2, 1)?;
            let k = count(s, r + 0x30)?;
            s.records(r + 0x38, k, 8, 3)?;
        }
    }
    Ok(name)
}

/// `ZBarrierDef` (0x300; `zm_factory`'s `zm_factory_debris`): the name,
/// then six boards (0x70 each from 0x40) whose models, effects and sounds
/// are references to assets loaded earlier on the maps read. Inline
/// pointers past the name are not read yet.
pub fn zbarrier(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::ZBarrier.header_size(), &[0])?;
    s.string(h)
}
