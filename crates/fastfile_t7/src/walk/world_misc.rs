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
/// 0x88 D* (8 each, count at 0x80)
/// ```
pub fn xcam(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::XCam.header_size(), &[0, 8, 0x48, 0x50, 0x60, 0x88])?;
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
    let n = s.u8_at(h + 0x14)? as usize;
    if let Some(t) = s.records(h + 0x50, n, 32, 7)? {
        for i in 0..n {
            let b = t + 32 * i;
            let frames = count(s, b + 8)?;
            s.records(b + 0x10, frames, 48, 3)?;
        }
    }
    let n = count(s, h + 0x24)?;
    s.records(h + 0x60, n, 8, 3)?;
    let n = count(s, h + 0x80)?;
    s.records(h + 0x88, n, 8, 3)?;
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
/// counts in between. Loaded in that order.
pub fn sanim(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, crate::AssetType::SAnim.header_size(), &[0, 8, 0x18])?;
    let name = s.string(h)?;
    super::gfxworld::stream_wrapped(s, h + 8)?;
    let n = s.u16_at(h + 0x64)? as usize;
    s.records(h + 0x18, n, 28, 3)?;
    Ok(name)
}
