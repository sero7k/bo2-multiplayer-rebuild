//! Animations (`XAnimParts`).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::ptr::Ptr;
use crate::stream::VIRTUAL;
use crate::walk::{DeltaRec, Span, XAnimRec};

/// The delta translation: 8 bytes (size, smallTrans), then either the
/// frames header (mins, size, frames pointer: 32 bytes) and its indices
/// and frames, or one frame (3 floats).
fn delta_trans(
    w: &mut Walker,
    field: usize,
    big_indices: bool,
) -> Result<Option<(u16, bool, Span)>> {
    if !w.s.ptr_at(field)?.is_inline() {
        return Ok(None);
    }
    if w.s.ptr_at(field)? == Ptr::Insert {
        w.s.insert_slot();
    }
    let start = w.st(8, 3)?;
    let size = w.s.u16_at(start)?;
    let small = w.s.u8_at(start + 2)? != 0;
    if size != 0 {
        let head = w.st(32, 0)?;
        let idx = if big_indices { 2 } else { 1 } * (size as usize + 1);
        w.st(idx, 0)?;
        let frames_ptr = head + 24;
        if w.s.ptr_at(frames_ptr)?.is_inline() {
            if small {
                w.st(3 * (size as usize + 1), 0)?;
            } else {
                w.st(6 * (size as usize + 1), 3)?;
            }
        }
    } else {
        w.st(12, 0)?;
    }
    let end = w.s.pos();
    Ok(Some((
        size,
        small,
        Span {
            pos: start,
            len: end - start,
        },
    )))
}

/// The delta rotations: 8 bytes (size), then either the frames pointer (8)
/// and its indices and frames (`frame_len` each), or one frame.
fn delta_quat(
    w: &mut Walker,
    field: usize,
    big_indices: bool,
    frame_len: usize,
) -> Result<Option<(u16, Span)>> {
    if !w.s.ptr_at(field)?.is_inline() {
        return Ok(None);
    }
    if w.s.ptr_at(field)? == Ptr::Insert {
        w.s.insert_slot();
    }
    let start = w.st(8, 3)?;
    let size = w.s.u16_at(start)?;
    if size != 0 {
        let head = w.st(8, 0)?;
        let idx = if big_indices { 2 } else { 1 } * (size as usize + 1);
        w.st(idx, 0)?;
        if w.s.ptr_at(head)?.is_inline() {
            w.st(frame_len * (size as usize + 1), 3)?;
        }
    } else {
        w.st(frame_len, 0)?;
    }
    let end = w.s.pos();
    Ok(Some((
        size,
        Span {
            pos: start,
            len: end - start,
        },
    )))
}

pub(super) fn xanim(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::XAnimParts as X;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + X::name)?.unwrap_or_default();
    let u16_at = |w: &Walker, o: usize| w.s.u16_at(h + o);
    let mut rec = XAnimRec {
        name,
        data_byte_count: u16_at(w, X::dataByteCount)?,
        data_short_count: u16_at(w, X::dataShortCount)?,
        data_int_count: u16_at(w, X::dataIntCount)?,
        random_data_byte_count: u16_at(w, X::randomDataByteCount)?,
        random_data_int_count: u16_at(w, X::randomDataIntCount)?,
        numframes: u16_at(w, X::numframes)?,
        flags: w.s.u8_at(h + X::flags)?,
        notify_count: w.s.u8_at(h + X::notifyCount)?,
        asset_type: w.s.u8_at(h + X::assetType)?,
        ik_type: w.s.u8_at(h + X::ikType)?,
        random_data_short_count: w.s.u32_at(h + X::randomDataShortCount)?,
        index_count: w.s.u32_at(h + X::indexCount)?,
        framerate: w.s.f32_at(h + X::framerate)?,
        frequency: w.s.f32_at(h + X::frequency)?,
        ..Default::default()
    };
    for i in 0..10 {
        rec.bone_count[i] = w.s.u8_at(h + X::boneCount + i)?;
    }
    let bones = rec.bone_count[9] as usize;
    if let Some(at) = w.blob(h + X::names, 4 * bones, 3)? {
        rec.names = (0..bones)
            .map(|i| w.s.u32_at(at + 4 * i))
            .collect::<Result<_>>()?;
    }
    let nn = rec.notify_count as usize;
    if let Some(at) = w.blob(h + X::notify, 8 * nn, 3)? {
        rec.notify = (0..nn)
            .map(|i| Ok((w.s.u32_at(at + 8 * i)?, w.s.f32_at(at + 8 * i + 4)?)))
            .collect::<Result<_>>()?;
    }
    let big = rec.numframes >= 256;
    if let Some(dp) = w.inline_array(h + X::deltaPart, L::XAnimDeltaPart::SIZE, 3)? {
        let trans = delta_trans(w, dp + L::XAnimDeltaPart::trans, big)?;
        let quat2 = delta_quat(w, dp + L::XAnimDeltaPart::quat2, big, 4)?;
        let quat = delta_quat(w, dp + L::XAnimDeltaPart::quat, big, 8)?;
        rec.delta = Some(DeltaRec { trans, quat2, quat });
    }
    let span = |pos: Option<usize>, len: usize| pos.map(|pos| Span { pos, len });
    let n = rec.data_byte_count as usize;
    rec.data_byte = span(w.blob(h + X::dataByte, n, 0)?, n);
    let n = 2 * rec.data_short_count as usize;
    rec.data_short = span(w.blob(h + X::dataShort, n, 1)?, n);
    let n = 4 * rec.data_int_count as usize;
    rec.data_int = span(w.blob(h + X::dataInt, n, 3)?, n);
    let n = 2 * rec.random_data_short_count as usize;
    rec.random_data_short = span(w.blob(h + X::randomDataShort, n, 1)?, n);
    let n = rec.random_data_byte_count as usize;
    rec.random_data_byte = span(w.blob(h + X::randomDataByte, n, 0)?, n);
    let n = 4 * rec.random_data_int_count as usize;
    rec.random_data_int = span(w.blob(h + X::randomDataInt, n, 3)?, n);
    let n = rec.index_count as usize * if big { 2 } else { 1 };
    rec.indices = span(w.blob(h + X::indices, n, if big { 1 } else { 0 })?, n);
    // AW's additions: the secondary animation's name, blend shapes and
    // scripted viewmodel data.
    w.xstr(h + X::secondaryName)?;
    let shapes = u16_at(w, X::blendShapeCount)? as usize;
    w.blob(h + X::blendShapeWeightNames, 4 * shapes, 3)?;
    w.blob(h + X::blendShapeCoefficientMagnitudes, 3 * shapes, 0)?;
    let mut keys = 0usize;
    if let Some(at) = w.blob(h + X::numberOfBlendShapeKeys, 2 * shapes, 1)? {
        for i in 0..shapes {
            keys += w.s.u16_at(at + 2 * i)? as usize;
        }
    }
    w.blob(h + X::blendShapeKeys, 2 * keys, 1)?;
    w.blob(h + X::compressedBlendShapeCoefficients, 2 * (keys + 2 * shapes), 1)?;
    w.blob(
        h + X::blendShapeWeights,
        4 * shapes * (rec.numframes as usize + 1),
        3,
    )?;
    w.asset(AssetType::XAnimParts, h + X::alternativeAnim)?;
    w.blob(h + X::svAmimData, L::XAnimScriptedViewmodelAnimData::SIZE, 3)?;
    w.s.pop();
    w.cap.xanims.push(rec);
    Ok(AssetId {
        ty: AssetType::XAnimParts,
        index: (w.cap.xanims.len() - 1) as u32,
    })
}
