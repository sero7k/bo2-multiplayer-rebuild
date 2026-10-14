//! Raw files and streaming info (x64-zt `shared/assets/rawfile.hpp`,
//! `iw7/assets/streaming_info.cpp`).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{RawFileRec, Span};

pub(super) fn raw_file(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::RawFile as R;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + R::name)?.unwrap_or_default();
    let compressed_len = w.s.u32_at(h + R::compressedLen)?;
    let len = w.s.u32_at(h + R::len)?;
    let n = if compressed_len != 0 { compressed_len as usize } else { len as usize + 1 };
    let data = w.blob(h + R::buffer, n, 0)?.map(|pos| Span { pos, len: n });
    w.s.pop();
    w.cap.raw_files.push(RawFileRec { name, compressed_len, len, data });
    Ok(AssetId { ty: AssetType::RawFile, index: (w.cap.raw_files.len() - 1) as u32 })
}

/// `DB_StreamingInfo_CalcUInts`: the u32s of a 4-bit flag per item.
fn flag_words(count: usize) -> usize {
    (4 * count + 31) >> 5
}

/// A string table: pointers (aligned 8), then each string.
fn strings(w: &mut Walker, field: usize, n: usize) -> Result<()> {
    if let Some(table) = w.inline_array(field, 8 * n, 7)? {
        for i in 0..n {
            w.xstr(table + 8 * i)?;
        }
    }
    Ok(())
}

/// `AlwaysloadedFlags`: images, their flags, strings, their flags,
/// materials, their flags.
fn always_loaded(w: &mut Walker, f: usize) -> Result<()> {
    use L::AlwaysloadedFlags as A;
    let images = w.s.u32_at(f + A::imageCount)? as usize;
    if let Some(table) = w.inline_array(f + A::images, 8 * images, 7)? {
        for i in 0..images {
            w.asset(AssetType::Image, table + 8 * i)?;
        }
    }
    w.blob(f + A::imageFlags, 4 * flag_words(images), 3)?;
    let strs = w.s.u32_at(f + A::stringCount)? as usize;
    strings(w, f + A::strings, strs)?;
    w.blob(f + A::stringFlags, 4 * flag_words(strs), 3)?;
    let mats = w.s.u32_at(f + A::materialCount)? as usize;
    if let Some(table) = w.inline_array(f + A::materials, 8 * mats, 7)? {
        for i in 0..mats {
            w.asset(AssetType::Material, table + 8 * i)?;
        }
    }
    w.blob(f + A::materialFlags, 4 * flag_words(mats), 3)?;
    Ok(())
}

pub(super) fn streaming_info(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::StreamingInfo as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    always_loaded(w, h + S::flags)?;
    let models = w.s.u32_at(h + S::modelCount)? as usize;
    use L::AlwaysloadedFlagsPerModel as P;
    if std::env::var_os("IW7_DEBUG").is_some() {
        eprintln!("streaming_info {name}: per-model ptr {:?} virt before {:#x}", w.s.ptr_at(h + S::perModelFlags)?, w.s.offset(VIRTUAL));
    }
    if let Some(table) = w.inline_array(h + S::perModelFlags, P::SIZE * models, 7)? {
        if std::env::var_os("IW7_DEBUG").is_some() {
            eprintln!("  table at file {table:#x} virt now {:#x}", w.s.offset(VIRTUAL));
        }
        for i in 0..models {
            let m = table + P::SIZE * i;
            w.asset(AssetType::XModel, m + P::model)?;
            always_loaded(w, m + P::flags)?;
        }
    }
    for (count, field) in [(S::forcedStubAssetCount, S::forcedStubAssets), (S::transientFileCount, S::transientFiles), (S::transientAssetCount, S::transientAssets)] {
        let n = w.s.u32_at(h + count)? as usize;
        if let Some(table) = w.inline_array(h + field, 16 * n, 7)? {
            for i in 0..n {
                w.xstr(table + 16 * i)?;
            }
        }
    }
    let n = w.s.u32_at(h + S::stringCount)? as usize;
    strings(w, h + S::strings, n)?;
    let sets = w.s.u32_at(h + S::transientSets__count)? as usize;
    strings(w, h + S::transientSets__names, sets)?;
    w.blob(h + S::transientSets__masks, 4 * sets, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::StreamingInfo, name))
}
