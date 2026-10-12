//! Small assets: names, raw files, string tables, physics, tracers,
//! vehicles.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::{SCRIPT, VIRTUAL};
use crate::walk::{OtherRec, RawFileRec, Span};

pub(super) fn named_only(w: &mut Walker, ty: AssetType, h: usize, name_off: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = w.xstr(h + name_off)?.unwrap_or_default();
    w.s.pop();
    Ok(w.name_only(ty, name))
}

pub(super) fn net_const_strings(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::NetConstStrings as N;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + N::name)?.unwrap_or_default();
    let count = w.s.u32_at(h + N::entryCount)? as usize;
    if let Some(table) = w.inline_array(h + N::stringList, 8 * count, 7)? {
        for i in 0..count {
            w.xstr(table + 8 * i)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::NetConstStrings, name))
}

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

pub(super) fn script_file(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ScriptFile as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    w.s.push(SCRIPT);
    let compressed = w.s.u32_at(h + S::compressedLen)? as usize;
    let bytecode = w.s.u32_at(h + S::bytecodeLen)? as usize;
    w.blob(h + S::buffer, compressed, 0)?;
    w.blob(h + S::bytecode, bytecode, 0)?;
    w.s.pop();
    w.s.pop();
    Ok(w.name_only(AssetType::ScriptFile, name))
}

pub(super) fn string_table(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::StringTable as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    let n = (w.s.u32_at(h + S::columnCount)? as usize) * (w.s.u32_at(h + S::rowCount)? as usize);
    if let Some(values) = w.inline_array(h + S::values, L::StringTableCell::SIZE * n, 3)? {
        for i in 0..n {
            w.xstr(values + L::StringTableCell::SIZE * i + L::StringTableCell::string)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::StringTable, name))
}

pub(super) fn lua_file(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::LuaFile as F;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + F::name)?.unwrap_or_default();
    let len = w.s.u32_at(h + F::len)? as usize;
    w.blob(h + F::buffer, len, 15)?;
    w.s.pop();
    Ok(w.name_only(AssetType::LuaFile, name))
}

pub(super) fn localize(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::LocalizeEntry as E;
    w.s.push(VIRTUAL);
    w.xstr(h + E::value)?;
    let name = w.xstr(h + E::name)?.unwrap_or_default();
    w.s.pop();
    Ok(w.name_only(AssetType::LocalizeEntry, name))
}

pub(super) fn phys_preset(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PhysPreset as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    w.xstr(h + P::sndAliasPrefix)?;
    w.s.pop();
    Ok(w.name_only(AssetType::PhysPreset, name))
}

/// A `BrushWrapper` (physics collision brush).
fn brush_wrapper(w: &mut Walker, field: usize) -> Result<()> {
    use L::BrushWrapper as B;
    let Some(b) = w.inline_array(field, B::SIZE, 3)? else { return Ok(()) };
    let numsides = w.s.u16_at(b + B::brush__numsides)? as usize;
    if let Some(sides) = w.inline_array(b + B::brush__sides, L::cbrushside_t::SIZE * numsides, 0)? {
        for i in 0..numsides {
            let plane = sides + L::cbrushside_t::SIZE * i + L::cbrushside_t::plane;
            w.blob(plane, L::cplane_s::SIZE, 3)?;
        }
    }
    let edges = w.s.u32_at(b + B::totalEdgeCount)? as usize;
    w.blob(b + B::brush__baseAdjacentSide, edges, 0)?;
    w.blob(b + B::planes, L::cplane_s::SIZE * numsides, 3)?;
    Ok(())
}

pub(super) fn phys_collmap(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PhysCollmap as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let count = w.s.u32_at(h + P::count)? as usize;
    if let Some(geoms) = w.inline_array(h + P::geoms, L::PhysGeomInfo::SIZE * count, 3)? {
        for i in 0..count {
            brush_wrapper(w, geoms + L::PhysGeomInfo::SIZE * i + L::PhysGeomInfo::brushWrapper)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::PhysCollmap, name))
}

pub(super) fn tracer(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::TracerDef as T;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + T::name)?.unwrap_or_default();
    w.asset(AssetType::Material, h + T::material)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Tracer, name))
}

pub(super) fn colorization_set(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ColorizationSet as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::name)?.unwrap_or_default();
    for f in [C::colorization, C::colorizationAlternate, C::colorizationLow, C::colorizationMid, C::colorizationHigh] {
        w.asset(AssetType::Colorization, h + f)?;
    }
    w.asset(AssetType::Material, h + C::alternateMaskMaterial)?;
    w.s.pop();
    Ok(w.name_only(AssetType::ColorizationSet, name))
}

pub(super) fn light_def(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::GfxLightDef as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + D::name)?.unwrap_or_default();
    w.asset(AssetType::Image, h + D::attenuation__image)?;
    w.asset(AssetType::Image, h + D::cucoloris__image)?;
    w.s.pop();
    Ok(w.name_only(AssetType::LightDef, name))
}

/// A sound named by a weapon or vehicle (`SndAliasCustom`): an 8-byte
/// name slot (VIRTUAL, aligned 8) then the name.
pub(super) fn sound_custom(w: &mut Walker, field: usize) -> Result<Option<String>> {
    match w.s.ptr_at(field)? {
        crate::Ptr::Follow | crate::Ptr::Insert => {
            if w.s.ptr_at(field)? == crate::Ptr::Insert {
                w.s.insert_slot();
            }
            let slot = w.st(8, 7)?;
            w.xstr(slot)
        }
        crate::Ptr::Ref { block: VIRTUAL, offset } => {
            // A reference to an earlier name slot: its name.
            let Some(at) = w.vmap.file_pos(offset as u64) else { return Ok(None) };
            match w.s.ptr_at(at)? {
                crate::Ptr::Ref { block: VIRTUAL, offset } => Ok(w.string_at(offset as u64)),
                _ => Ok(w.string_at(offset as u64 + 8)),
            }
        }
        _ => Ok(None),
    }
}

pub(super) fn vehicle(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::VehicleDef as V;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + V::name)?.unwrap_or_default();
    w.xstr(h + V::useHintString)?;
    w.xstr(h + V::vehPhysDef__physPresetName)?;
    w.asset(AssetType::PhysPreset, h + V::vehPhysDef__physPreset)?;
    w.xstr(h + V::vehPhysDef__accelGraphName)?;
    w.asset(AssetType::Fx, h + V::vehHelicopterGroundFx)?;
    w.asset(AssetType::Fx, h + V::vehHelicopterGroundWaterFx)?;
    w.xstr(h + V::turretWeaponName)?;
    // (The zone tools never write the turret's weapon; retail vehicles
    // carry it inline.)
    w.asset(AssetType::Weapon, h + V::turretWeapon)?;
    sound_custom(w, h + V::turretSpinSnd)?;
    sound_custom(w, h + V::turretStopSnd)?;
    w.asset(AssetType::Fx, h + V::trophyExplodeFx)?;
    w.asset(AssetType::Fx, h + V::trophyFlashFx)?;
    for f in [V::compassFriendlyIcon, V::compassEnemyIcon, V::compassFriendlyAltIcon, V::compassEnemyAltIcon] {
        w.asset(AssetType::Material, h + f)?;
    }
    for f in [
        V::idleLowSnd,
        V::idleHighSnd,
        V::engineLowSnd,
        V::engineHighSnd,
        V::idleLowSndAlt,
        V::idleHighSndAlt,
        V::engineLowSndAlt,
        V::engineHighSndAlt,
        V::turretSpinSndAlt,
        V::turretStopSndAlt,
        V::engineStartUpSnd,
        V::engineShutdownSnd,
        V::engineIdleSnd,
        V::engineSustainSnd,
        V::engineRampUpSnd,
        V::engineRampDownSnd,
        V::suspensionSoftSnd,
        V::suspensionHardSnd,
        V::collisionSnd,
        V::speedSnd,
    ] {
        sound_custom(w, h + f)?;
    }
    w.xstr(h + V::surfaceSndPrefix)?;
    for i in 0..31 {
        sound_custom(w, h + V::surfaceSnds + 8 * i)?;
    }
    w.xstr(h + V::soundTriggerOverrideZone)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Vehicle, name))
}

#[allow(dead_code)]
fn _unused(_: OtherRec) {}
