//! Weapons (x64-zt `iw7/assets/weapondef.cpp`): the complete def, its
//! `WeaponDef`, and the arrays both own, in the writer's order.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{AnimPackageRec, AttachmentRec, SfxPackageRec, Span, WeaponField, WeaponRec};
use WeaponField::{Complete, Def};

/// An `FxCombinedDef`: an effect pointer whose type byte says which kind.
pub(super) fn fx_combined(w: &mut Walker, field: usize) -> Result<Option<AssetId>> {
    let ty = if w.s.u8_at(field + 8)? == 1 { AssetType::Vfx } else { AssetType::Fx };
    w.asset(ty, field)
}

/// An asset pointer at `base + f`, recorded by field.
fn put(w: &mut Walker, rec: &mut WeaponRec, ty: AssetType, base: usize, f: usize, at: fn(usize) -> WeaponField) -> Result<()> {
    if let Some(id) = w.asset(ty, base + f)? {
        rec.assets.push((at(f), id));
    }
    Ok(())
}

fn put_str(w: &mut Walker, rec: &mut WeaponRec, base: usize, f: usize, at: fn(usize) -> WeaponField) -> Result<()> {
    if let Some(s) = w.xstr(base + f)? {
        rec.strings.push((at(f), s));
    }
    Ok(())
}

fn put_fx(w: &mut Walker, rec: &mut WeaponRec, base: usize, f: usize, at: fn(usize) -> WeaponField) -> Result<()> {
    if let Some(id) = fx_combined(w, base + f)? {
        rec.assets.push((at(f), id));
    }
    Ok(())
}

/// A pointer to `n` asset pointers (aligned 8).
pub(super) fn asset_array(w: &mut Walker, ty: AssetType, field: usize, n: usize) -> Result<Vec<Option<AssetId>>> {
    let mut out = Vec::new();
    if let Some(table) = w.inline_array(field, 8 * n, 7)? {
        for i in 0..n {
            out.push(w.asset(ty, table + 8 * i)?);
        }
    }
    Ok(out)
}

fn weapon_def(w: &mut Walker, d: usize, parent: usize, rec: &mut WeaponRec) -> Result<()> {
    use AssetType as T;
    use L::WeaponDef as D;
    if std::env::var_os("IW7_WDUMP").is_some() {
        for o in (0..D::SIZE).step_by(8) {
            let v = w.s.u64_at(d + o)?;
            if v != 0 {
                eprintln!("  def+{o:5} {v:#018x}");
            }
        }
    }
    put_str(w, rec, d, D::szOverlayName, Def)?;
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
        put(w, rec, T::XModel, d, f, Def)?;
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
        for (i, id) in asset_array(w, T::XModel, d + f, complex)?.into_iter().enumerate() {
            if let Some(id) = id {
                rec.assets.push((Def(f + 8 * i), id));
            }
        }
    }
    for f in [D::camoWorldModelMaterialOverride, D::camoViewModelMaterialOverride] {
        for i in 0..16 {
            put(w, rec, T::Material, d, f + 8 * i, Def)?;
        }
    }
    for f in [D::szXAnims, D::szXAnimsRightHanded, D::szXAnimsLeftHanded] {
        put(w, rec, T::AnimPackage, d, f, Def)?;
    }
    put_str(w, rec, d, D::szModeName, Def)?;
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
    put(w, rec, T::Material, d, D::unkMaterial, Def)?;
    put(w, rec, T::VfxPackage, d, D::vfxPackage, Def)?;
    put(w, rec, T::SfxPackage, d, D::sfxPackage, Def)?;
    for f in [D::reticleCenter, D::reticleSide, D::reticleOnePiece] {
        put(w, rec, T::Material, d, f, Def)?;
    }
    for f in [D::worldClipModel, D::rocketModel, D::knifeModel, D::worldKnifeModel] {
        put(w, rec, T::XModel, d, f, Def)?;
    }
    for f in [D::hudIcon, D::pickupIcon, D::dangerIcon, D::throwBackIcon, D::ammoCounterIcon] {
        put(w, rec, T::Material, d, f, Def)?;
    }
    put_str(w, rec, d, D::szAmmoName, Def)?;
    put_str(w, rec, d, D::szSharedAmmoCapName, Def)?;
    for f in [D::overlay__shader, D::overlay__shaderLowRes, D::overlay__shaderEMP, D::overlay__shaderEMPLowRes] {
        put(w, rec, T::Material, d, f, Def)?;
    }
    put(w, rec, T::XModel, d, D::projectileModel, Def)?;
    put_fx(w, rec, d, D::projExplosionEffect, Def)?;
    put_fx(w, rec, d, D::projDudEffect, Def)?;
    w.blob(d + D::parallelBounce, 4 * 64, 3)?;
    w.blob(d + D::perpendicularBounce, 4 * 64, 3)?;
    for f in [D::projBodyEffect, D::projTrailEffect, D::projBeaconEffect, D::projIgnitionEffect] {
        put_fx(w, rec, d, f, Def)?;
    }
    // The writer sizes the original knots by the complete def's counts.
    for i in 0..2 {
        put_str(w, rec, d, D::accuracyGraphName + 8 * i, Def)?;
        let n = w.s.u16_at(parent + L::WeaponCompleteDef::accuracyGraphKnotCount + 2 * i)? as usize;
        w.blob(d + D::originalAccuracyGraphKnots + 8 * i, 8 * n, 3)?;
    }
    put_str(w, rec, d, D::szUseHintString, Def)?;
    put_str(w, rec, d, D::dropHintString, Def)?;
    put_str(w, rec, d, D::szScript, Def)?;
    w.blob(d + D::locationDamageMultipliers, 4 * 22, 3)?;
    w.blob(d + D::hitLocPriorityMap, 22, 0)?;
    put(w, rec, T::Rumble, d, D::fireRumble, Def)?;
    put(w, rec, T::Rumble, d, D::meleeImpactRumble, Def)?;
    put(w, rec, T::Tracer, d, D::tracerType, Def)?;
    put(w, rec, T::Tracer, d, D::overchargeTracerType, Def)?;
    put(w, rec, T::Laser, d, D::laserTypeFriendly, Def)?;
    put(w, rec, T::Laser, d, D::laserTypeEnemy, Def)?;
    put_str(w, rec, d, D::turretOverheatSound, Def)?;
    put_fx(w, rec, d, D::turretOverheatEffect, Def)?;
    put(w, rec, T::Rumble, d, D::turretBarrelSpinRumble, Def)?;
    put_str(w, rec, d, D::turretBarrelSpinMaxSnd, Def)?;
    for f in [D::turretBarrelSpinUpSnd, D::turretBarrelSpinDownSnd] {
        for i in 0..4 {
            put_str(w, rec, d, f + 8 * i, Def)?;
        }
    }
    put_str(w, rec, d, D::missileConeSoundAlias, Def)?;
    put_str(w, rec, d, D::missileConeSoundAliasAtBase, Def)?;
    for f in [
        D::shieldDeployGesture,
        D::shieldFireWeapGesture,
        D::shieldDeployWhileFiring,
        D::shieldRetractWhileFiring,
        D::shieldBashGesture,
    ] {
        put(w, rec, T::Gesture, d, f, Def)?;
    }
    put_fx(w, rec, d, D::shieldMeleeFx, Def)?;
    put(w, rec, T::Rumble, d, D::shieldHitRumble, Def)?;
    put(w, rec, T::Rumble, d, D::shieldMissRumble, Def)?;
    for f in [D::gestureAnimation, D::gesturePullback, D::gestureThrow, D::gestureDetonate] {
        put(w, rec, T::Gesture, d, f, Def)?;
    }
    put_str(w, rec, d, D::unknownString1, Def)?;
    put_str(w, rec, d, D::unknownString2, Def)?;
    put_fx(w, rec, d, D::cameraFireEffect, Def)?;
    put_fx(w, rec, d, D::chargeMeterEffect, Def)?;
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
        put_str(w, rec, d, f, Def)?;
    }
    put(w, rec, T::Rumble, d, D::chargeRumble, Def)?;
    put(w, rec, T::XModel, d, D::stowOffsetModel, Def)?;
    put_fx(w, rec, d, D::deployEffect, Def)?;
    for (base, ty) in [
        (D::animOverrides, T::AnimPackage),
        (D::sfxOverrides, T::SfxPackage),
        (D::vfxOverrides, T::VfxPackage),
    ] {
        for i in 0..21 {
            put(w, rec, ty, d, base + 24 * i + 8, Def)?;
            put(w, rec, ty, d, base + 24 * i + 16, Def)?;
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
    if std::env::var_os("IW7_WNAME").is_some() {
        let p = w.s.u64_at(h + C::name)?;
        let r = p & 0xFFFF_FFFF;
        let l = w.local(r.wrapping_sub(1));
        eprintln!("weapon name ptr {p:#x} local {l:#x} file {:?} name {name:?} display {:?}", w.vmap.file_pos(l), w.s.u64_at(h + C::szDisplayName)?);
    }
    let mut rec = WeaponRec {
        name: name.clone(),
        complete: Span { pos: h, len: C::SIZE },
        ..Default::default()
    };
    if let Some(d) = w.inline_array(h + C::weapDef, L::WeaponDef::SIZE, 7)? {
        rec.def = Some(Span { pos: d, len: L::WeaponDef::SIZE });
        weapon_def(w, d, h, &mut rec)?;
    }
    rec.display_name = w.xstr(h + C::szDisplayName)?.unwrap_or_default();
    put_str(w, &mut rec, h, C::szLootTable, Complete)?;
    if let Some(at) = w.blob(h + C::hideTags, 4 * 32, 3)? {
        rec.hide_tags = (0..32).map(|i| w.s.u32_at(at + 4 * i)).collect::<Result<_>>()?;
    }
    for (k, (count, field)) in [
        (C::numAttachments, C::attachments),
        (C::numAttachments2, C::attachments2),
        (C::numAttachments3, C::attachments3),
        (C::numAttachments4, C::attachments4),
        (C::numAttachments5, C::attachments5),
        (C::numAttachments6, C::attachments6),
    ]
    .into_iter()
    .enumerate()
    {
        let n = w.s.u32_at(h + count)? as usize;
        rec.attachments[k] = asset_array(w, T::Attachment, h + field, n)?;
    }
    let n = w.s.u32_at(h + C::numNotetrackSoundMappings)? as usize;
    w.blob(h + C::notetrackSoundMapKeys, 4 * n, 3)?;
    w.blob(h + C::notetrackSoundMapValues, 4 * n, 3)?;
    notetrack_entries(w, h + C::numNotetrackOverrides, h + C::notetrackOverrides)?;
    notetrack_entries(w, h + C::numNotetrackSuitEntries, h + C::notetrackSuitEntries)?;
    put_str(w, &mut rec, h, C::szAltWeaponName, Complete)?;
    put(w, &mut rec, T::Material, h, C::killIcon, Complete)?;
    put(w, &mut rec, T::Material, h, C::dpadIcon, Complete)?;
    for i in 0..2 {
        let n = w.s.u16_at(h + C::accuracyGraphKnotCount + 2 * i)? as usize;
        w.blob(h + C::accuracyGraphKnots + 8 * i, 8 * n, 3)?;
    }
    for f in [C::szAdsrBaseSetting, C::szLUIWeaponInfoWidgetName, C::szLUIWeaponInfoWidgetTag, C::szLUICrosshairWidget] {
        put_str(w, &mut rec, h, f, Complete)?;
    }
    w.s.pop();
    w.cap.weapons.push(rec);
    Ok(AssetId { ty: T::Weapon, index: (w.cap.weapons.len() - 1) as u32 })
}

/// `NUM_WEAP_ANIMS`.
const WEAPON_ANIMS: usize = 260;

pub(super) fn anim_package(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponAnimPackage as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    if std::env::var_os("IW7_APDUMP").is_some() {
        eprintln!("anim package {name} pos {:#x} header {:x?}", w.s.pos(), (0..8).map(|i| w.s.u64_at(h + 8 * i).unwrap_or(0)).collect::<Vec<_>>());
    }
    let anims = asset_array(w, AssetType::XAnimParts, h + P::anims, WEAPON_ANIMS)?;
    w.blob(h + P::timers, L::WeaponAnimPackageStateTimers::SIZE, 3)?;
    w.blob(h + P::footstep, L::FootstepAnims::SIZE, 3)?;
    w.s.pop();
    w.cap.anim_packages.push(AnimPackageRec { name, anims });
    Ok(AssetId { ty: AssetType::AnimPackage, index: (w.cap.anim_packages.len() - 1) as u32 })
}

pub(super) fn sfx_package(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponSFXPackage as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let len = L::WeaponSFXPackageSounds::SIZE;
    let mut sounds = Vec::new();
    if let Some(s) = w.inline_array(h + P::sounds, len, 7)? {
        for o in (0..len).step_by(8) {
            if let Some(t) = w.xstr(s + o)? {
                sounds.push((o, t));
            }
        }
    }
    w.xstr(h + P::szAdsrBaseSetting)?;
    w.xstr(h + P::szWeapSndReflectionClass)?;
    w.s.pop();
    w.cap.sfx_packages.push(SfxPackageRec { name, sounds });
    Ok(AssetId { ty: AssetType::SfxPackage, index: (w.cap.sfx_packages.len() - 1) as u32 })
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
    let mut rec = AttachmentRec {
        name: name.clone(),
        record: Span { pos: h, len: A::SIZE },
        ..Default::default()
    };
    rec.display_name = w.xstr(h + A::szDisplayName)?.unwrap_or_default();
    for f in [A::worldModelCamo, A::viewModelCamo] {
        if let Some(id) = w.asset(T::XModel, h + f)? {
            rec.assets.push((f, id));
        }
    }
    let variations = w.s.u32_at(h + A::numModelVariations)? as usize;
    for (i, id) in asset_array(w, T::XModel, h + A::worldModelVariations, variations)?.into_iter().enumerate() {
        if let Some(id) = id {
            rec.assets.push((A::worldModelVariations + 8 * i, id));
        }
    }
    for (i, id) in asset_array(w, T::XModel, h + A::viewModelVariations, variations)?.into_iter().enumerate() {
        if let Some(id) = id {
            rec.assets.push((A::viewModelVariations + 8 * i, id));
        }
    }
    let n = w.s.u32_at(h + A::numWorldModelCamoMaterials)? as usize;
    for (i, id) in asset_array(w, T::Material, h + A::worldModelCamoMaterials, n)?.into_iter().enumerate() {
        if let Some(id) = id {
            rec.assets.push((A::worldModelCamoMaterials + 8 * i, id));
        }
    }
    let n = w.s.u32_at(h + A::numViewModelCamoMaterials)? as usize;
    for (i, id) in asset_array(w, T::Material, h + A::viewModelCamoMaterials, n)?.into_iter().enumerate() {
        if let Some(id) = id {
            rec.assets.push((A::viewModelCamoMaterials + 8 * i, id));
        }
    }
    let n = w.s.u32_at(h + A::numReticles)? as usize;
    for (i, id) in asset_array(w, T::Reticle, h + A::reticles, n)?.into_iter().enumerate() {
        if let Some(id) = id {
            rec.assets.push((A::reticles + 8 * i, id));
        }
    }
    let n = w.s.u32_at(h + A::numAltReticles)? as usize;
    for (i, id) in asset_array(w, T::Reticle, h + A::altReticles, n)?.into_iter().enumerate() {
        if let Some(id) = id {
            rec.assets.push((A::altReticles + 8 * i, id));
        }
    }
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
                    if let Some(id) = w.asset(ty, s + o)? {
                        rec.assets.push((field + o, id));
                    }
                }
                Str(o) => {
                    if let Some(t) = w.xstr(s + o)? {
                        rec.strings.push((field + o, t));
                    }
                }
                Fx(o) => {
                    if let Some(id) = fx_combined(w, s + o)? {
                        rec.assets.push((field + o, id));
                    }
                }
            }
        }
    }
    w.blob(h + A::locationDamage, 4 * 22, 3)?;
    for f in [A::szAltModeName, A::szLUIWeaponInfoWidgetName, A::szLUIWeaponInfoWidgetTag] {
        if let Some(t) = w.xstr(h + f)? {
            rec.strings.push((f, t));
        }
    }
    w.s.pop();
    w.cap.attachments.push(rec);
    Ok(AssetId { ty: T::Attachment, index: (w.cap.attachments.len() - 1) as u32 })
}
