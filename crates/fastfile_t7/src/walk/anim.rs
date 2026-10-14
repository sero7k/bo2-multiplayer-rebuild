//! Animations (`XAnimParts`, 0xf8; x64-zt `t7/structs.hpp` layout).
//!
//! ```text
//! 0x00 char* name; 0x08 u32 randomDataByteCount, dataShortCount, extraChannelDataCount,
//!      dataByteCount, dataIntCount, randomDataIntCount; 0x20 u16 numframes, numBones
//! 0x48 u32 randomDataShortCount, indexCount; 0x60 u32 numIKPitchLayers, numIKPitchBones
//! 0x68 u32* names; 0x70 u8* dataByte; 0x78 i16* dataShort; 0x80 i32* dataInt;
//! 0x88 i16* randomDataShort; 0x90 u8* randomDataByte; 0x98 i32* randomDataInt;
//! 0xa0 u8* extraChannelData; 0xa8 indices (u8/u16); 0xb0 ikPitchLayers (8 each);
//! 0xb8 ikPitchBones (28 each); 0xc0 notify, 0xc8 u8 count; 0xd0 startupNotify,
//! 0xd8 u8 count; 0xe0 shutdownNotify, 0xe8 u8 count (16 each); 0xf0 XAnimDeltaPart*
//! ```
//!
//! Stream order (read on retail `mp_common`; the sizes add up to the next
//! asset exactly): name, bone names (`numBones` u32), the three notify
//! arrays, the delta part `{ trans; quat2; quat }` (24) and its parts, then
//! the data arrays in field order: dataByte, dataShort, dataInt,
//! randomDataShort, randomDataByte, randomDataInt, extraChannelData,
//! indices, IK pitch layers, IK pitch bones.
//!
//! Delta parts (indices u8 when `numframes < 256`, else u16):
//! ```text
//! XAnimPartTrans       u16 size; u8 smallTrans; size == 0: vec3 frame0 at 4 (16 bytes)
//!                      else: vec3 mins, size at 4; frames* at 0x20; (size + 1) indices at 0x28,
//!                      then the frames: (size + 1) x (smallTrans ? 3 : 6) bytes
//! XAnimDeltaPartQuat2  u16 size; size == 0: XQuat2 frame0 at 8 (12 bytes)
//!                      else: frames* at 8; (size + 1) indices at 0x10; frames (size + 1) x 4
//! XAnimDeltaPartQuat   u16 size; size == 0: XQuat frame0 at 8 (16 bytes)
//!                      else: frames* at 8; (size + 1) indices at 0x10; frames (size + 1) x 8
//! ```

use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};
use crate::stream::Stream;

const NOTIFY: usize = 16;
const IK_LAYER: usize = 8;
const IK_BONE: usize = 28;

/// One delta part component at `field`: `head` bytes before the indices
/// when animated, `still` bytes when not, `frame` bytes per frame.
fn delta_component(s: &mut Stream, field: usize, index: usize, still: usize, head: usize, frame: Option<usize>) -> Result<()> {
    if !s.ptr_at(field)?.is_inline() {
        return s.array(field, 0, 0).map(drop);
    }
    // The size decides how much of the struct the stream holds.
    let at = s.pos();
    let size = s.u16_at(at)? as usize;
    let len = if size == 0 { still } else { head + (size + 1) * index };
    let t = s.array(field, len, 7)?.ok_or(ReadError::Invalid("delta part", field))?;
    if size > 0 {
        let per = match frame {
            Some(n) => n,
            // Translations: 3 bytes per frame when small, else 6.
            None => {
                if s.u8_at(t + 2)? != 0 {
                    3
                } else {
                    6
                }
            }
        };
        let frames_at = if frame.is_none() { 0x20 } else { 8 };
        s.array(t + frames_at, (size + 1) * per, 1)?;
    }
    Ok(())
}

pub fn xanim(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let mut known: Vec<usize> = (0x68..0xf8).step_by(8).collect();
    known.push(0);
    s.no_loads(h, AssetType::XAnimParts.header_size(), &known)?;
    let name = s.string(h)?;
    let frames = s.u16_at(h + 0x20)? as usize;
    let bones = s.u16_at(h + 0x22)? as usize;
    let index = if frames >= 256 { 2 } else { 1 };
    s.records(h + 0x68, bones, 4, 3)?;
    for (ptr, count) in [(0xc0, 0xc8), (0xd0, 0xd8), (0xe0, 0xe8)] {
        let n = s.u8_at(h + count)? as usize;
        s.records(h + ptr, n, NOTIFY, 3)?;
    }
    if let Some(d) = s.array(h + 0xf0, 24, 7)? {
        delta_component(s, d, index, 0x10, 0x28, None)?;
        delta_component(s, d + 8, index, 12, 0x10, Some(4))?;
        delta_component(s, d + 16, index, 16, 0x10, Some(8))?;
    }
    let u32_at = |o: usize| s.u32_at(h + o).map(|v| v as usize);
    let arrays = [
        (0x70, u32_at(0x14)?, 0),         // dataByte
        (0x78, u32_at(0x0c)? * 2, 1),     // dataShort
        (0x80, u32_at(0x18)? * 4, 3),     // dataInt
        (0x88, u32_at(0x48)? * 2, 1),     // randomDataShort
        (0x90, u32_at(0x08)?, 0),         // randomDataByte
        (0x98, u32_at(0x1c)? * 4, 3),    // randomDataInt
        (0xa0, u32_at(0x10)?, 0),        // extraChannelData
        (0xa8, u32_at(0x4c)? * index, 1), // indices
        (0xb0, u32_at(0x60)? * IK_LAYER, 3),
        (0xb8, u32_at(0x64)? * IK_BONE, 3),
    ];
    for (ptr, len, mask) in arrays {
        s.array(h + ptr, len, mask)?;
    }
    Ok(name)
}
