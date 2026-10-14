//! Weapons (x64-zt `iw7/assets/weapondef.cpp`): the complete def, its
//! `WeaponDef`, and the arrays both own, in the writer's order.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

/// An `FxCombinedDef`: an effect pointer whose type byte says which kind.
pub(super) fn fx_combined(w: &mut Walker, field: usize) -> Result<()> {
    let ty = if w.s.u8_at(field + 8)? == 1 { AssetType::Vfx } else { AssetType::Fx };
    w.asset(ty, field)?;
    Ok(())
}

/// A pointer to `n` asset pointers (aligned 8).
pub(super) fn asset_array(w: &mut Walker, ty: AssetType, field: usize, n: usize) -> Result<()> {
    if let Some(table) = w.inline_array(field, 8 * n, 7)? {
        for i in 0..n {
            w.asset(ty, table + 8 * i)?;
        }
    }
    Ok(())
}

fn weapon_def(w: &mut Walker, d: usize, parent: usize) -> Result<()> {
    use AssetType as T;
    use L::WeaponDef as D;
    let model = |w: &mut Walker, f: usize| w.asset(T::XModel, d + f).map(|_| ());
    let material = |w: &mut Walker, f: usize| w.asset(T::Material, d + f).map(|_| ());
    if std::env::var_os("IW7_WDUMP").is_some() {
        for o in (0..D::SIZE).step_by(8) {
            let v = w.s.u64_at(d + o)?;
            if v != 0 {
                eprintln!("  def+{o:5} {v:#018x}");
            }
        }
    }
    w.xstr(d + D::szOverlayName)?;
    for f in [
        D::gunXModel,
        D::gunXModelLeftHand,
        D::gunXModelRightHand,
        D::defaultViewModel,
        D::defaultWorldModelLeftHand,
        D::defaultWorldModelRightHand,
        D::worldModel,
        D::worldXModelLeftHand,
        D::worldXModelRightHand,
        D::defaultWorldModel,
        D::playerShadowModel,
        D::playerShadowModelLeftHand,
        D::playerShadowModelRightHand,
        D::handXModel,
    ] {
        model(w, f)?;
    }
    let complex = w.s.u32_at(d + D::numComplexModels)? as usize;
    for f in [
        D::complexWorldModel,
        D::complexWorldModelLeftHand,
        D::complexWorldModelRightHand,
        D::complexGunXModel,
        D::complexGunXModelLeftHand,
        D::complexGunXModelRightHand,
    ] {
        asset_array(w, T::XModel, d + f, complex)?;
    }
    for f in [D::camoWorldModelMaterialOverride, D::camoViewModelMaterialOverride] {
        for i in 0..16 {
            material(w, f + 8 * i)?;
        }
    }
    for f in [D::szXAnims, D::szXAnimsRightHanded, D::szXAnimsLeftHanded] {
        w.asset(T::AnimPackage, d + f)?;
    }
    w.xstr(d + D::szModeName)?;
    w.blob(d + D::notetrackRumbleMapKeys, 4 * 16, 3)?;
    w.blob(d + D::notetrackRumbleMapValues, 4 * 16, 3)?;
    w.blob(d + D::notetrackFXMapKeys, 4 * 16, 3)?;
    let fx_len = L::FxCombinedDef::SIZE;
    if let Some(table) = w.inline_array(d + D::notetrackFXMapValues, fx_len * 16, 7)? {
        for i in 0..16 {
            fx_combined(w, table + fx_len * i)?;
        }
    }
    w.blob(d + D::notetrackFXMapTagValues, 4 * 16, 3)?;
    material(w, D::unkMaterial)?;
    w.asset(T::VfxPackage, d + D::vfxPackage)?;
    w.asset(T::SfxPackage, d + D::sfxPackage)?;
    for f in [D::reticleCenter, D::reticleSide, D::reticleOnePiece] {
        material(w, f)?;
    }
    for f in [D::worldClipModel, D::rocketModel, D::knifeModel, D::worldKnifeModel] {
        model(w, f)?;
    }
    for f in [D::hudIcon, D::pickupIcon, D::dangerIcon, D::throwBackIcon, D::ammoCounterIcon] {
        material(w, f)?;
    }
    w.xstr(d + D::szAmmoName)?;
    w.xstr(d + D::szSharedAmmoCapName)?;
    for f in [D::overlay__shader, D::overlay__shaderLowRes, D::overlay__shaderEMP, D::overlay__shaderEMPLowRes] {
        material(w, f)?;
    }
    model(w, D::projectileModel)?;
    fx_combined(w, d + D::projExplosionEffect)?;
    fx_combined(w, d + D::projDudEffect)?;
    w.blob(d + D::parallelBounce, 4 * 64, 3)?;
    w.blob(d + D::perpendicularBounce, 4 * 64, 3)?;
    for f in [D::projBodyEffect, D::projTrailEffect, D::projBeaconEffect, D::projIgnitionEffect] {
        fx_combined(w, d + f)?;
    }
    // The writer sizes the original knots by the complete def's counts.
    for i in 0..2 {
        w.xstr(d + D::accuracyGraphName + 8 * i)?;
        let n = w.s.u16_at(parent + L::WeaponCompleteDef::accuracyGraphKnotCount + 2 * i)? as usize;
        w.blob(d + D::originalAccuracyGraphKnots + 8 * i, 8 * n, 3)?;
    }
    w.xstr(d + D::szUseHintString)?;
    w.xstr(d + D::dropHintString)?;
    w.xstr(d + D::szScript)?;
    w.blob(d + D::locationDamageMultipliers, 4 * 22, 3)?;
    w.blob(d + D::hitLocPriorityMap, 22, 0)?;
    w.asset(T::Rumble, d + D::fireRumble)?;
    w.asset(T::Rumble, d + D::meleeImpactRumble)?;
    w.asset(T::Tracer, d + D::tracerType)?;
    w.asset(T::Tracer, d + D::overchargeTracerType)?;
    w.asset(T::Laser, d + D::laserTypeFriendly)?;
    w.asset(T::Laser, d + D::laserTypeEnemy)?;
    w.xstr(d + D::turretOverheatSound)?;
    fx_combined(w, d + D::turretOverheatEffect)?;
    w.asset(T::Rumble, d + D::turretBarrelSpinRumble)?;
    w.xstr(d + D::turretBarrelSpinMaxSnd)?;
    for f in [D::turretBarrelSpinUpSnd, D::turretBarrelSpinDownSnd] {
        for i in 0..4 {
            w.xstr(d + f + 8 * i)?;
        }
    }
    w.xstr(d + D::missileConeSoundAlias)?;
    w.xstr(d + D::missileConeSoundAliasAtBase)?;
    for f in [
        D::shieldDeployGesture,
        D::shieldFireWeapGesture,
        D::shieldDeployWhileFiring,
        D::shieldRetractWhileFiring,
        D::shieldBashGesture,
    ] {
        w.asset(T::Gesture, d + f)?;
    }
    fx_combined(w, d + D::shieldMeleeFx)?;
    w.asset(T::Rumble, d + D::shieldHitRumble)?;
    w.asset(T::Rumble, d + D::shieldMissRumble)?;
    for f in [D::gestureAnimation, D::gesturePullback, D::gestureThrow, D::gestureDetonate] {
        w.asset(T::Gesture, d + f)?;
    }
    w.xstr(d + D::unknownString1)?;
    w.xstr(d + D::unknownString2)?;
    fx_combined(w, d + D::cameraFireEffect)?;
    fx_combined(w, d + D::chargeMeterEffect)?;
    for f in [
        D::chargeUpSound,
        D::chargeDownSound,
        D::chargeUpSoundPlayer,
        D::chargeDownSoundPlayer,
        D::chargeDownToUpSound,
        D::chargeDownToUpSoundPlayer,
        D::chargeUpToDownSound,
        D::chargeUpToDownSoundPlayer,
        D::chargeMaxSound,
        D::chargeMaxSoundPlayer,
    ] {
        w.xstr(d + f)?;
    }
    w.asset(T::Rumble, d + D::chargeRumble)?;
    model(w, D::stowOffsetModel)?;
    fx_combined(w, d + D::deployEffect)?;
    for (base, ty) in [
        (D::animOverrides, T::AnimPackage),
        (D::sfxOverrides, T::SfxPackage),
        (D::vfxOverrides, T::VfxPackage),
    ] {
        for i in 0..21 {
            w.asset(ty, d + base + 24 * i + 8)?;
            w.asset(ty, d + base + 24 * i + 16)?;
        }
    }
    Ok(())
}

/// `NoteTrackToSoundEntry` / `NoteTrackToSuitSoundEntry` arrays (24 bytes:
/// count, a u32, keys, values).
fn notetrack_entries(w: &mut Walker, count: usize, field: usize) -> Result<()> {
    let n = w.s.u32_at(count)? as usize;
    if let Some(table) = w.inline_array(field, 24 * n, 7)? {
        for i in 0..n {
            let e = table + 24 * i;
            let m = w.s.u32_at(e)? as usize;
            w.blob(e + 8, 4 * m, 3)?;
            w.blob(e + 16, 4 * m, 3)?;
        }
    }
    Ok(())
}

pub(super) fn weapon(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::WeaponCompleteDef as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::name)?.unwrap_or_default();
    if let Some(d) = w.inline_array(h + C::weapDef, L::WeaponDef::SIZE, 7)? {
        weapon_def(w, d, h)?;
    }
    w.xstr(h + C::szDisplayName)?;
    w.xstr(h + C::szLootTable)?;
    w.blob(h + C::hideTags, 4 * 32, 3)?;
    for (count, field) in [
        (C::numAttachments, C::attachments),
        (C::numAttachments2, C::attachments2),
        (C::numAttachments3, C::attachments3),
        (C::numAttachments4, C::attachments4),
        (C::numAttachments5, C::attachments5),
        (C::numAttachments6, C::attachments6),
    ] {
        let n = w.s.u32_at(h + count)? as usize;
        asset_array(w, T::Attachment, h + field, n)?;
    }
    let n = w.s.u32_at(h + C::numNotetrackSoundMappings)? as usize;
    w.blob(h + C::notetrackSoundMapKeys, 4 * n, 3)?;
    w.blob(h + C::notetrackSoundMapValues, 4 * n, 3)?;
    notetrack_entries(w, h + C::numNotetrackOverrides, h + C::notetrackOverrides)?;
    notetrack_entries(w, h + C::numNotetrackSuitEntries, h + C::notetrackSuitEntries)?;
    w.xstr(h + C::szAltWeaponName)?;
    w.asset(T::Material, h + C::killIcon)?;
    w.asset(T::Material, h + C::dpadIcon)?;
    for i in 0..2 {
        let n = w.s.u16_at(h + C::accuracyGraphKnotCount + 2 * i)? as usize;
        w.blob(h + C::accuracyGraphKnots + 8 * i, 8 * n, 3)?;
    }
    for f in [C::szAdsrBaseSetting, C::szLUIWeaponInfoWidgetName, C::szLUIWeaponInfoWidgetTag, C::szLUICrosshairWidget] {
        w.xstr(h + f)?;
    }
    w.s.pop();
    Ok(w.name_only(T::Weapon, name))
}

/// `NUM_WEAP_ANIMS`.
const WEAPON_ANIMS: usize = 260;

pub(super) fn anim_package(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponAnimPackage as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    asset_array(w, AssetType::XAnimParts, h + P::anims, WEAPON_ANIMS)?;
    w.blob(h + P::timers, L::WeaponAnimPackageStateTimers::SIZE, 3)?;
    w.blob(h + P::footstep, L::FootstepAnims::SIZE, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::AnimPackage, name))
}

pub(super) fn sfx_package(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponSFXPackage as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let len = L::WeaponSFXPackageSounds::SIZE;
    if let Some(s) = w.inline_array(h + P::sounds, len, 7)? {
        for o in (0..len).step_by(8) {
            w.xstr(s + o)?;
        }
    }
    w.xstr(h + P::szAdsrBaseSetting)?;
    w.xstr(h + P::szWeapSndReflectionClass)?;
    w.s.pop();
    Ok(w.name_only(AssetType::SfxPackage, name))
}

pub(super) fn vfx_package(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponVFXPackage as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let len = L::WeaponVFXPackageDefs::SIZE;
    let fx = L::FxCombinedDef::SIZE;
    if let Some(d) = w.inline_array(h + P::vfx, len, 7)? {
        for o in (0..len).step_by(fx) {
            fx_combined(w, d + o)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::VfxPackage, name))
}

/// What one attachment section (`Att*`) points at.
enum Ref {
    Asset(AssetType, usize),
    Str(usize),
    Fx(usize),
}

pub(super) fn attachment(w: &mut Walker, h: usize) -> Result<AssetId> {
    use AssetType as T;
    use L::WeaponAttachment as A;
    use Ref::*;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + A::name)?.unwrap_or_default();
    if std::env::var_os("IW7_ADUMP").is_some() {
        eprintln!("attachment {name} header {h:#x} pos {:#x}", w.s.pos());
        for o in (0..A::SIZE).step_by(8) {
            let v = w.s.u64_at(h + o)?;
            if v != 0 {
                eprintln!("  att+{o:4} {v:#018x}");
            }
        }
    }
    w.xstr(h + A::szDisplayName)?;
    w.asset(T::XModel, h + A::worldModelCamo)?;
    w.asset(T::XModel, h + A::viewModelCamo)?;
    let variations = w.s.u32_at(h + A::numModelVariations)? as usize;
    asset_array(w, T::XModel, h + A::worldModelVariations, variations)?;
    asset_array(w, T::XModel, h + A::viewModelVariations, variations)?;
    let n = w.s.u32_at(h + A::numWorldModelCamoMaterials)? as usize;
    asset_array(w, T::Material, h + A::worldModelCamoMaterials, n)?;
    let n = w.s.u32_at(h + A::numViewModelCamoMaterials)? as usize;
    asset_array(w, T::Material, h + A::viewModelCamoMaterials, n)?;
    let n = w.s.u32_at(h + A::numReticles)? as usize;
    asset_array(w, T::Reticle, h + A::reticles, n)?;
    let n = w.s.u32_at(h + A::numAltReticles)? as usize;
    asset_array(w, T::Reticle, h + A::altReticles, n)?;
    let m = |f| Asset(T::Material, f);
    // The sections in the writer's order: field, size, alignment, pointers.
    let sections: Vec<(usize, usize, u64, Vec<Ref>)> = vec![
        (A::ammogeneral, L::AttAmmoGeneral::SIZE, 3, vec![]),
        (A::tracer, L::AttAmmoTracer::SIZE, 7, vec![Asset(T::Tracer, L::AttAmmoTracer::tracerType)]),
        (A::penetration, L::AttPenetration::SIZE, 3, vec![]),
        (A::sight, L::AttSight::SIZE, 3, vec![]),
        (A::holdBreath, L::AttHoldBreath::SIZE, 0, vec![]),
        (A::reload, L::AttReload::SIZE, 0, vec![]),
        (A::addOns, L::AttAddOns::SIZE, 0, vec![]),
        (
            A::general,
            L::AttGeneral::SIZE,
            7,
            vec![
                m(L::AttGeneral::reticleCenter),
                m(L::AttGeneral::reticleSide),
                m(L::AttGeneral::reticleOnePiece),
                Str(L::AttGeneral::szLUICrosshairWidget),
            ],
        ),
        (
            A::laser,
            L::AttLaser::SIZE,
            7,
            vec![Asset(T::Laser, L::AttLaser::laserTypeFriendly), Asset(T::Laser, L::AttLaser::laserTypeEnemy)],
        ),
        (A::aimAssist, L::AttAimAssist::SIZE, 3, vec![]),
        (A::targetAssist, L::AttTargetAssist::SIZE, 3, vec![]),
        (A::unkAtt01, L::AttUnknown01::SIZE, 7, vec![m(L::AttUnknown01::material)]),
        (A::ammunition, L::AttAmmunition::SIZE, 3, vec![]),
        (A::damage, L::AttDamage::SIZE, 3, vec![]),
        (A::idleSettings, L::AttIdleSettings::SIZE, 3, vec![]),
        (A::adsSettings, L::AttADSSettings::SIZE, 3, vec![]),
        (A::adsSettingsMain, L::AttADSSettings::SIZE, 3, vec![]),
        (A::scopeDriftSettings, L::AttScopeDriftSettings::SIZE, 3, vec![]),
        (A::scopeDriftSettingsMain, L::AttScopeDriftSettings::SIZE, 3, vec![]),
        (A::hipSpread, L::AttHipSpread::SIZE, 3, vec![]),
        (A::slideSpread, L::AttSlideSpread::SIZE, 3, vec![]),
        (A::gunKick, L::AttGunKick::SIZE, 3, vec![]),
        (A::viewKick, L::AttViewKick::SIZE, 3, vec![]),
        (A::adsOverlay, L::AttADSOverlay::SIZE, 7, vec![m(0), m(8), m(16), m(24)]),
        (A::outline, L::AttOutline::SIZE, 3, vec![]),
        (
            A::ui,
            L::AttUI::SIZE,
            7,
            vec![
                m(L::AttUI::hudIcon),
                m(L::AttUI::pickupIcon),
                m(L::AttUI::dangerIcon),
                m(L::AttUI::throwBackIcon),
                m(L::AttUI::dpadIcon),
                m(L::AttUI::ammoCounterIcon),
            ],
        ),
        (
            A::rumbles,
            L::AttRumbles::SIZE,
            7,
            vec![Asset(T::Rumble, L::AttRumbles::fireRumble), Asset(T::Rumble, L::AttRumbles::meleeImpactRumble)],
        ),
        (A::projectile, L::AttProjectile::SIZE, 7, {
            use L::AttProjectile as P;
            vec![
                Asset(T::XModel, P::projectileModel),
                Fx(P::projExplosionEffect),
                Str(P::projExplosionSound),
                Fx(P::projDudEffect),
                Str(P::projDudSound),
                Fx(P::projBodyEffect),
                Fx(P::projTrailEffect),
                Fx(P::projBeaconEffect),
                Fx(P::projIgnitionEffect),
                Str(P::projIgnitionSound),
            ]
        }),
        (A::charged, L::AttCharged::SIZE, 7, {
            use L::AttCharged as C;
            vec![
                Fx(C::chargeMeterEffect),
                Str(C::chargeUpSound),
                Str(C::chargeDownSound),
                Str(C::chargeDownToUpSound),
                Str(C::chargeUpToDownSound),
                Str(C::chargeMaxSound),
                Str(C::chargeUpSoundPlayer),
                Str(C::chargeDownSoundPlayer),
                Str(C::chargeDownToUpSoundPlayer),
                Str(C::chargeUpToDownSoundPlayer),
                Str(C::chargeMaxSoundPlayer),
                Asset(T::Rumble, C::chargeRumble),
            ]
        }),
        (A::adsAltSwitch, L::AdsAltSwitch::SIZE, 3, vec![]),
        (A::regeneration, L::AttRegeneration::SIZE, 3, vec![]),
        (A::movement, L::AttMovement::SIZE, 3, vec![]),
        (A::burst, L::AttBurst::SIZE, 3, vec![]),
    ];
    for (field, size, align, refs) in sections {
        let Some(s) = w.inline_array(h + field, size, align)? else {
            continue;
        };
        for r in refs {
            match r {
                Asset(ty, o) => {
                    w.asset(ty, s + o)?;
                }
                Str(o) => {
                    w.xstr(s + o)?;
                }
                Fx(o) => fx_combined(w, s + o)?,
            }
        }
    }
    w.blob(h + A::locationDamage, 4 * 22, 3)?;
    w.xstr(h + A::szAltModeName)?;
    w.xstr(h + A::szLUIWeaponInfoWidgetName)?;
    w.xstr(h + A::szLUIWeaponInfoWidgetTag)?;
    w.s.pop();
    Ok(w.name_only(T::Attachment, name))
}
