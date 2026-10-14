//! Suits and suit anim packages: no x64-zt writer, so the order is
//! inferred from `SuitDef` / `SuitAnimPackage` (x64-zt `iw7/structs.hpp`)
//! and checked against the retail data. Suits load their anim packages and
//! scriptable "inserted" (FFFD), sounds are strings, rumbles assets.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

fn dbg() -> bool {
    std::env::var_os("IW7_SUITDUMP").is_some()
}

pub(super) fn suit(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::SuitDef as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + D::name)?.unwrap_or_default();
    if dbg() {
        eprintln!("suit {name} header {h:#x} pos {:#x}", w.s.pos());
    }
    for f in [D::doubleJump_sound, D::doubleJump_soundPlayer, D::doubleJump_releaseSound, D::doubleJump_releaseSoundPlayer] {
        w.xstr(h + f)?;
    }
    for f in [D::animPackage, D::animPackageL, D::animPackageR, D::animPackageRelaxed, D::animPackageSafe] {
        w.asset(T::SuitAnimPackage, h + f)?;
    }
    w.asset(T::Scriptable, h + D::scriptableDef)?;
    w.asset(T::Rumble, h + D::groundPound_activationRumble)?;
    w.xstr(h + D::groundPound_activationSound)?;
    w.xstr(h + D::groundPound_activationSoundPlayer)?;
    w.asset(T::Rumble, h + D::groundPound_landingRumble)?;
    w.xstr(h + D::groundPound_landingSound)?;
    w.xstr(h + D::groundPound_landingSoundPlayer)?;
    for f in [D::landing_rumbleLowHeight, D::landing_rumbleMediumHeight, D::landing_rumbleHighHeight, D::landing_rumbleExtremeHeight, D::footstep_rumble] {
        w.asset(T::Rumble, h + f)?;
    }
    if dbg() {
        eprintln!("  suit end pos {:#x}", w.s.pos());
    }
    w.s.pop();
    Ok(w.name_only(T::Suit, name))
}

/// `SuitAnimPackage`: its name, then each of the six per-weapon-class
/// overrides' anims (`SuitAnimEntry`s, then each anim) and gestures
/// (`SuitGestureEntry`s, then each gesture).
pub(super) fn suit_anim_package(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::SuitAnimPackage as P;
    use L::SuitAnimPerWeapClass as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    if dbg() {
        eprintln!("suitanimpackage {name} header {h:#x} pos {:#x}", w.s.pos());
    }
    for k in 0..6 {
        let c = h + P::animOverrides + C::SIZE * k;
        let n = w.s.u32_at(c + C::numAnims)? as usize;
        if let Some(t) = w.inline_array(c + C::anims, 16 * n, 7)? {
            for i in 0..n {
                w.asset(AssetType::XAnimParts, t + 16 * i)?;
            }
        }
        let n = w.s.u32_at(c + C::numGestures)? as usize;
        if let Some(t) = w.inline_array(c + C::gestures, 16 * n, 7)? {
            for i in 0..n {
                w.asset(AssetType::Gesture, t + 16 * i)?;
            }
        }
    }
    if dbg() {
        eprintln!("  suitanimpackage end pos {:#x}", w.s.pos());
    }
    w.s.pop();
    Ok(w.name_only(AssetType::SuitAnimPackage, name))
}
