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
        if std::env::var_os("IW7_DEBUG").is_some() {
            eprintln!("alwaysloaded images {images} table virt {:#x}", w.vmap.virt_of(table).unwrap_or(0));
        }
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
    if std::env::var_os("IW7_DEBUG").is_some() {
        eprintln!("streaming_info header {h:#x} virt {:#x}", w.s.offset(VIRTUAL));
        for o in (0..S::SIZE).step_by(8) {
            eprintln!("  si+{o:3} {:#018x}", w.s.u64_at(h + o)?);
        }
    }
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
            if std::env::var_os("IW7_DEBUG").is_some() {
                eprintln!("si table field {field} n {n} at {table:#x} virt {:#x}", w.vmap.virt_of(table).unwrap_or(0));
                for i in 0..n.min(4) {
                    eprintln!("   {:#018x} {:#018x}", w.s.u64_at(table + 16 * i)?, w.s.u64_at(table + 16 * i + 8)?);
                }
            }
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

pub(super) fn net_const_strings(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::NetConstStrings as N;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + N::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + N::entryCount)? as usize;
    strings(w, h + N::stringList, n)?;
    w.s.pop();
    Ok(w.name_only(AssetType::NetConstStrings, name))
}

pub(super) fn light_def(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::GfxLightDef as G;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + G::name)?.unwrap_or_default();
    w.xstr(h + G::iesProfile)?;
    w.s.pop();
    Ok(w.name_only(AssetType::LightDef, name))
}

pub(super) fn tracer(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::TracerDef as T;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + T::name)?.unwrap_or_default();
    w.asset(AssetType::Material, h + T::material)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Tracer, name))
}

/// A script: its name, then its source (compressed) and bytecode in the
/// SCRIPT block.
pub(super) fn script_file(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ScriptFile as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    w.s.push(crate::stream::SCRIPT);
    let compressed = w.s.u32_at(h + S::compressedLen)? as usize;
    w.blob(h + S::buffer, compressed, 0)?;
    let bytecode = w.s.u32_at(h + S::bytecodeLen)? as usize;
    w.blob(h + S::bytecode, bytecode, 0)?;
    w.s.pop();
    w.s.pop();
    Ok(w.name_only(AssetType::ScriptFile, name))
}

pub(super) fn rumble(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::RumbleInfo as R;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + R::name)?.unwrap_or_default();
    w.asset(AssetType::RumbleGraph, h + R::highRumbleGraph)?;
    w.asset(AssetType::RumbleGraph, h + R::lowRumbleGraph)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Rumble, name))
}

/// Assets whose only pointer is their name (rumble graphs).
pub(super) fn name_only(w: &mut Walker, ty: AssetType, h: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = w.xstr(h)?.unwrap_or_default();
    w.s.pop();
    Ok(w.name_only(ty, name))
}

pub(super) fn laser(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::LaserDef as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + D::name)?.unwrap_or_default();
    w.asset(AssetType::Material, h + D::laserMaterial)?;
    w.asset(AssetType::Material, h + D::laserLightMaterial)?;
    super::weapon::fx_combined(w, h + D::beamEffect)?;
    super::weapon::fx_combined(w, h + D::laserEndEffect)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Laser, name))
}

pub(super) fn reticle(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ReticleDef as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + D::name)?.unwrap_or_default();
    w.xstr(h + D::widgetName)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Reticle, name))
}

/// `GESTURE_ASSET_NUM`.
const GESTURE_ANIMS: usize = 18;

pub(super) fn gesture(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::Gesture as G;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + G::name)?.unwrap_or_default();
    super::weapon::asset_array(w, AssetType::XAnimParts, h + G::anims, GESTURE_ANIMS)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Gesture, name))
}
