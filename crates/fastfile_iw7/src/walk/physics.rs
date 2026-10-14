//! Physics assets, their sound and effect event assets, and procedural
//! bones (x64-zt `iw7/assets/{physics_asset,physics_sfx_event,
//! physics_vfx_event}.cpp`; procedural bones have no writer there: their
//! arrays are read in field order).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

pub(super) fn physics_asset(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PhysicsAsset as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let havok = w.s.u32_at(h + P::havokDataSize)? as usize;
    w.blob(h + P::havokData, havok, 15)?;
    for (count, field, ty) in [
        (P::numSFXEventAssets, P::sfxEventAssets, AssetType::PhysicsSfxEventAsset),
        (P::numVFXEventAssets, P::vfxEventAssets, AssetType::PhysicsVfxEventAsset),
    ] {
        let n = w.s.u32_at(h + count)? as usize;
        if let Some(table) = w.inline_array(h + field, 8 * n, 7)? {
            for i in 0..n {
                w.asset(ty, table + 8 * i)?;
            }
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::PhysicsAsset, name))
}

/// A rules list (`{ int numrules; Rule* rules; }` at `field`).
fn rules(w: &mut Walker, field: usize, sfx: bool) -> Result<()> {
    let n = w.s.u32_at(field)? as usize;
    let len = if sfx { L::PhysicsSFXEventAssetRule::SIZE } else { L::PhysicsVFXEventAssetRule::SIZE };
    let Some(table) = w.inline_array(field + 8, len * n, 7)? else {
        return Ok(());
    };
    for i in 0..n {
        let r = table + len * i;
        let ty = w.s.u32_at(r)?;
        match ty {
            // A sound rule's aliases / an effect rule's effects.
            1 => {
                w.xstr(r + 8)?;
                w.xstr(r + 16)?;
            }
            2..=9 => {
                // The entries: their size and where their rules are.
                let (size, at) = if sfx && (3..=5).contains(&ty) { (40, 24) } else { (24, 8) };
                let m = w.s.u32_at(r + 8)? as usize;
                if let Some(entries) = w.inline_array(r + 16, size * m, 7)? {
                    for k in 0..m {
                        rules(w, entries + size * k + at, sfx)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn event_asset(w: &mut Walker, ty: AssetType, h: usize) -> Result<AssetId> {
    use L::PhysicsSFXEventAsset as E;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + E::name)?.unwrap_or_default();
    rules(w, h + E::rules, ty == AssetType::PhysicsSfxEventAsset)?;
    w.s.pop();
    Ok(w.name_only(ty, name))
}

pub(super) fn procedural_bones(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::XAnimProceduralBones as B;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + B::name)?.unwrap_or_default();
    if std::env::var_os("IW7_DEBUG").is_some() {
        eprintln!("procbones {name} pos {:#x} virt {:#x}", w.s.pos(), w.s.offset(VIRTUAL));
        for o in (0..B::SIZE).step_by(8) {
            eprintln!("  pb+{o:3} {:#018x}", w.s.u64_at(h + o)?);
        }
    }
    let constraints = w.s.u32_at(h + B::numConstraints)? as usize;
    w.blob(h + B::constraints, L::XAnimConstraint::SIZE * constraints, 3)?;
    let targets = w.s.u32_at(h + B::numTargetBones)? as usize;
    w.blob(h + B::targetBoneNames, 4 * targets, 3)?;
    let n1 = w.s.u32_at(h + B::unk01_count)? as usize;
    // Each 0x80-byte entry holds two string pointers (+96, +104; x64-zt
    // has them as padding), read after the array.
    if let Some(table) = w.inline_array(h + B::unk01, 0x80 * n1, 3)? {
        for i in 0..n1 {
            if std::env::var_os("IW7_DEBUG").is_some() {
                eprintln!("  pb entry {i} virt {:#x} ptrs {:x} {:x}", w.s.offset(VIRTUAL), w.s.u64_at(table + 0x80 * i + 96)?, w.s.u64_at(table + 0x80 * i + 104)?);
            }
            w.xstr(table + 0x80 * i + 96)?;
            w.xstr(table + 0x80 * i + 104)?;
        }
    }
    let dbg = std::env::var_os("IW7_DEBUG").is_some();
    if dbg {
        eprintln!("  pb after unk01 pos {:#x} virt {:#x}", w.s.pos(), w.s.offset(VIRTUAL));
    }
    let n2 = w.s.u32_at(h + B::unk02_count)? as usize;
    w.blob(h + B::unk02, 12 * n2, 3)?;
    if dbg {
        eprintln!("  pb after unk02 pos {:#x} virt {:#x}", w.s.pos(), w.s.offset(VIRTUAL));
    }
    let n3 = w.s.u32_at(h + B::unk03_count)? as usize;
    w.blob(h + B::unk03, 32 * n3, 3)?;
    if dbg {
        eprintln!("  pb after unk03 pos {:#x} virt {:#x}", w.s.pos(), w.s.offset(VIRTUAL));
    }
    w.s.pop();
    Ok(w.name_only(AssetType::XAnimProceduralBones, name))
}

/// Physics FX pipelines: a name and 68-byte entries.
pub(super) fn fx_pipeline(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PhysicsFXPipeline as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + P::unk__numUnk)? as usize;
    w.blob(h + P::unk__unk, 68 * n, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::PhysicsFxPipeline, name))
}

/// Physics FX shapes: a name and `size` vectors.
pub(super) fn fx_shape(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PhysicsFXShape as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + S::size)? as usize;
    w.blob(h + S::vecs, 12 * n, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::PhysicsFxShape, name))
}

pub(super) fn library(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PhysicsLibrary as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + P::havokDataSize)? as usize;
    w.blob(h + P::havokData, n, 15)?;
    w.s.pop();
    Ok(w.name_only(AssetType::PhysicsLibrary, name))
}
