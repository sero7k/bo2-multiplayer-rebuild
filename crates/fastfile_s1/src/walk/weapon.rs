//! Weapons (`WeaponDef`, the asset itself in AW) and attachments
//! (`WeaponAttachment`: a list of weapon fields it changes).
//!
//! The order is the `WeaponDef` struct's (the loader's); x64-zt's writer
//! follows it except where it is wrong (it writes `fireLastSound` twice for
//! `fireCustomSound`, swaps the melee hit/miss sounds and never writes
//! `camoViewModelMaterialOverride`).

use super::misc::sound_custom;
use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{AnimOverride, AttachmentField, AttachmentRec, Span, WeaponRec};

/// Weapon animations (`NUM_WEAP_ANIMS`).
pub const NUM_WEAP_ANIMS: usize = 180;
/// Surface types (bounce sounds and factors).
pub const SURF_TYPES: usize = 53;

/// An array of `n` asset pointers (VIRTUAL, aligned 8).
fn asset_array(
    w: &mut Walker,
    ty: AssetType,
    field: usize,
    n: usize,
) -> Result<Vec<Option<AssetId>>> {
    let mut out = Vec::new();
    if let Some(at) = w.inline_array(field, 8 * n, 7)? {
        for i in 0..n {
            out.push(w.asset(ty, at + 8 * i)?);
        }
    } else if let Some(v) = w.s.ptr_at(field)?.virtual_offset()
        && let Some(at) = w.vmap.file_pos(v)
    {
        for i in 0..n {
            out.push(
                w.s.ptr_at(at + 8 * i)?
                    .virtual_offset()
                    .and_then(|v| w.resolve_slot(ty, v)),
            );
        }
    }
    Ok(out)
}

fn script_strings(w: &mut Walker, field: usize, n: usize) -> Result<Vec<u32>> {
    match w.blob(field, 4 * n, 3)? {
        Some(at) => (0..n).map(|i| w.s.u32_at(at + 4 * i)).collect(),
        None => Ok(Vec::new()),
    }
}

/// `WAFieldType`s whose value is a string (names of assets too).
const STRING_FIELDS: [u8; 7] = [0, 10, 11, 12, 13, 15, 16];

pub(super) fn attachment(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponAttachment as A;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + A::szInternalName)?.unwrap_or_default();
    let display_name = w.xstr(h + A::szDisplayName)?.unwrap_or_default();
    let mut rec = AttachmentRec {
        name,
        display_name,
        ty: w.s.i32_at(h + A::r#type)?,
        weap_type: w.s.i32_at(h + A::weaponType)?,
        weap_class: w.s.i32_at(h + A::weapClass)?,
        record: Span {
            pos: h,
            len: A::SIZE,
        },
        ..Default::default()
    };
    rec.world_models = asset_array(w, AssetType::XModel, h + A::worldModels, 2)?;
    rec.view_models = asset_array(w, AssetType::XModel, h + A::viewModels, 2)?;
    rec.reticle_view_models = asset_array(w, AssetType::XModel, h + A::reticleViewModels, 32)?;
    w.blob(h + A::chargeInfo, L::AttChargeInfo::SIZE, 3)?;
    w.blob(h + A::hybridSettings, L::AttHybridSettings::SIZE, 3)?;
    let n = w.s.i32_at(h + A::numFields)?.max(0) as usize;
    let offsets = w.blob(h + A::fieldOffsets, 2 * n, 1)?;
    let f_len = L::WAField::SIZE;
    if let Some(at) = w.inline_array(h + A::fields, f_len * n, 3)? {
        for i in 0..n {
            let f = at + f_len * i;
            let ty = w.s.u8_at(f + L::WAField::r#type)?;
            let text = if STRING_FIELDS.contains(&ty) {
                w.xstr(f + L::WAField::parm)?
            } else {
                None
            };
            rec.fields.push(AttachmentField {
                index: w.s.u8_at(f + L::WAField::index)?,
                ty,
                code: w.s.u8_at(f + L::WAField::code)?,
                text,
                raw: w.s.u64_at(f + L::WAField::parm)?,
                offset: match offsets {
                    Some(o) => w.s.u16_at(o + 2 * i)?,
                    None => 0,
                },
            });
        }
    }
    w.s.pop();
    w.cap.attachments.push(rec);
    Ok(AssetId {
        ty: AssetType::Attachment,
        index: (w.cap.attachments.len() - 1) as u32,
    })
}

fn sound_field(w: &mut Walker, rec: &mut WeaponRec, d: usize, off: usize) -> Result<()> {
    if let Some(name) = sound_custom(w, d + off)? {
        rec.sounds.push((off, name));
    }
    Ok(())
}

fn fx_field(w: &mut Walker, rec: &mut WeaponRec, d: usize, off: usize) -> Result<()> {
    if let Some(id) = w.asset(AssetType::Fx, d + off)? {
        rec.fx.push((off, id));
    }
    Ok(())
}

fn model_field(w: &mut Walker, rec: &mut WeaponRec, d: usize, off: usize) -> Result<()> {
    if let Some(id) = w.asset(AssetType::XModel, d + off)? {
        rec.models.push((off, id));
    }
    Ok(())
}

fn material_field(w: &mut Walker, rec: &mut WeaponRec, d: usize, off: usize) -> Result<()> {
    if let Some(id) = w.asset(AssetType::Material, d + off)? {
        rec.materials.push((off, id));
    }
    Ok(())
}

fn string_field(w: &mut Walker, rec: &mut WeaponRec, d: usize, off: usize) -> Result<()> {
    if let Some(s) = w.xstr(d + off)? {
        rec.strings.push((off, s));
    }
    Ok(())
}

/// The sound pointers from `pickupSound` to `adsCrosshairEnemySound`, in
/// struct order.
const WEAPON_SOUNDS: [usize; 74] = {
    use L::WeaponDef as D;
    [
        D::pickupSound,
        D::pickupSoundPlayer,
        D::ammoPickupSound,
        D::ammoPickupSoundPlayer,
        D::projectileSound,
        D::pullbackSound,
        D::pullbackSoundPlayer,
        D::throwbackSound,
        D::throwbackSoundPlayer,
        D::fireSound,
        D::fireSoundPlayer,
        D::fireSoundPlayerAkimbo,
        D::fireMedSound,
        D::fireMedSoundPlayer,
        D::fireHighSound,
        D::fireHighSoundPlayer,
        D::fireLoopSound,
        D::fireLoopSoundPlayer,
        D::fireMedLoopSound,
        D::fireMedLoopSoundPlayer,
        D::fireHighLoopSound,
        D::fireHighLoopSoundPlayer,
        D::fireLoopEndPointSound,
        D::fireLoopEndPointSoundPlayer,
        D::fireStopSound,
        D::fireStopSoundPlayer,
        D::fireMedStopSound,
        D::fireMedStopSoundPlayer,
        D::fireHighStopSound,
        D::fireHighStopSoundPlayer,
        D::fireLastSound,
        D::fireLastSoundPlayer,
        D::fireFirstSound,
        D::fireFirstSoundPlayer,
        D::fireCustomSound,
        D::fireCustomSoundPlayer,
        D::emptyFireSound,
        D::emptyFireSoundPlayer,
        D::adsRequiredFireSoundPlayer,
        D::meleeSwipeSound,
        D::meleeSwipeSoundPlayer,
        D::meleeHitSound,
        D::meleeMissSound,
        D::meleeHitSoundPlayer,
        D::meleeMissSoundPlayer,
        D::rechamberSound,
        D::rechamberSoundPlayer,
        D::reloadSound,
        D::reloadSoundPlayer,
        D::reloadEmptySound,
        D::reloadEmptySoundPlayer,
        D::reloadStartSound,
        D::reloadStartSoundPlayer,
        D::reloadEndSound,
        D::reloadEndSoundPlayer,
        D::detonateSound,
        D::detonateSoundPlayer,
        D::nightVisionWearSound,
        D::nightVisionWearSoundPlayer,
        D::nightVisionRemoveSound,
        D::nightVisionRemoveSoundPlayer,
        D::altSwitchSound,
        D::altSwitchSoundPlayer,
        D::raiseSound,
        D::raiseSoundPlayer,
        D::firstRaiseSound,
        D::firstRaiseSoundPlayer,
        D::putawaySound,
        D::putawaySoundPlayer,
        D::scanSound,
        D::changeVariableZoomSound,
        D::adsUpSound,
        D::adsDownSound,
        D::adsCrosshairEnemySound,
    ]
};

pub(super) fn weapon(w: &mut Walker, d: usize) -> Result<AssetId> {
    use L::WeaponDef as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(d + D::szInternalName)?.unwrap_or_default();
    let mut rec = WeaponRec {
        name,
        def: Span {
            pos: d,
            len: D::SIZE,
        },
        ..Default::default()
    };
    rec.display_name = w.xstr(d + D::szDisplayName)?.unwrap_or_default();
    rec.alt_weapon_name = w.xstr(d + D::szAltWeaponName)?.unwrap_or_default();
    rec.gun_models = asset_array(w, AssetType::XModel, d + D::gunModel, 2)?;
    rec.hand_model = w.asset(AssetType::XModel, d + D::handModel)?;
    model_field(w, &mut rec, d, D::persistentArmXModel)?;
    string_field(w, &mut rec, d, D::lobWorldModelName)?;
    material_field(w, &mut rec, d, D::camoWorldModelMaterialOverride)?;
    material_field(w, &mut rec, d, D::camoViewModelMaterialOverride)?;
    rec.anims_right =
        asset_array(w, AssetType::XAnimParts, d + D::szXAnimsRightHanded, NUM_WEAP_ANIMS)?;
    rec.anims_left =
        asset_array(w, AssetType::XAnimParts, d + D::szXAnimsLeftHanded, NUM_WEAP_ANIMS)?;
    rec.hide_tags = script_strings(w, d + D::hideTags, 32)?;
    let n = w.s.u8_at(d + D::numAttachments)? as usize;
    rec.attachments = asset_array(w, AssetType::Attachment, d + D::attachments, n)?;
    rec.anims = asset_array(w, AssetType::XAnimParts, d + D::szXAnims, NUM_WEAP_ANIMS)?;
    let n = w.s.u8_at(d + D::numAnimOverrides)? as usize;
    let ao = L::AnimOverrideEntry::SIZE;
    if let Some(at) = w.inline_array(d + D::animOverrides, ao * n, 3)? {
        use L::AnimOverrideEntry as E;
        for i in 0..n {
            let e = at + ao * i;
            let override_anim = w.asset(AssetType::XAnimParts, e + E::overrideAnim)?;
            let altmode_anim = w.asset(AssetType::XAnimParts, e + E::altmodeAnim)?;
            rec.anim_overrides.push(AnimOverride {
                attachment1: w.s.u8_at(e + E::attachment1)? as u32,
                attachment2: w.s.u8_at(e + E::attachment2)? as u32,
                anim_index: w.s.u8_at(e + E::animTreeType)? as u32,
                override_anim,
                altmode_anim,
                anim_time: w.s.i32_at(e + E::animTime)?,
                altmode_time: w.s.i32_at(e + E::altTime)?,
            });
        }
    }
    let n = w.s.u8_at(d + D::numSoundOverrides)? as usize;
    let so = L::SoundOverrideEntry::SIZE;
    if let Some(at) = w.inline_array(d + D::soundOverrides, so * n, 3)? {
        use L::SoundOverrideEntry as E;
        for i in 0..n {
            let e = at + so * i;
            let a = sound_custom(w, e + E::overrideSound)?;
            let b = sound_custom(w, e + E::altmodeSound)?;
            rec.sound_overrides.push((
                w.s.u8_at(e + E::attachment1)? as u32,
                w.s.u8_at(e + E::attachment2)? as u32,
                a,
                b,
            ));
        }
    }
    let n = w.s.u8_at(d + D::numFXOverrides)? as usize;
    let fo = L::FXOverrideEntry::SIZE;
    if let Some(at) = w.inline_array(d + D::fxOverrides, fo * n, 3)? {
        for i in 0..n {
            let e = at + fo * i;
            w.asset(AssetType::Fx, e + L::FXOverrideEntry::overrideFX)?;
            w.asset(AssetType::Fx, e + L::FXOverrideEntry::altmodeFX)?;
        }
    }
    let n = w.s.u8_at(d + D::numReloadStateTimerOverrides)? as usize;
    w.blob(d + D::reloadOverrides, L::ReloadStateTimerEntry::SIZE * n, 3)?;
    let n = w.s.u8_at(d + D::numNotetrackOverrides)? as usize;
    let no = L::NoteTrackToSoundEntry::SIZE;
    if let Some(at) = w.inline_array(d + D::notetrackOverrides, no * n, 3)? {
        for i in 0..n {
            let e = at + no * i;
            w.blob(e + L::NoteTrackToSoundEntry::notetrackSoundMapKeys, 4 * 36, 3)?;
            w.blob(e + L::NoteTrackToSoundEntry::notetrackSoundMapValues, 4 * 36, 3)?;
        }
    }
    rec.notetrack_sound_keys = script_strings(w, d + D::notetrackSoundMapKeys, 36)?;
    rec.notetrack_sound_values = script_strings(w, d + D::notetrackSoundMapValues, 36)?;
    script_strings(w, d + D::notetrackRumbleMapKeys, 16)?;
    script_strings(w, d + D::notetrackRumbleMapValues, 16)?;
    script_strings(w, d + D::notetrackFXMapKeys, 16)?;
    asset_array(w, AssetType::Fx, d + D::notetrackFXMapValues, 16)?;
    script_strings(w, d + D::notetrackFXMapTagValues, 16)?;
    string_field(w, &mut rec, d, D::szAdsrBaseSetting)?;
    for f in [
        D::viewFlashEffect,
        D::viewBodyFlashEffect,
        D::worldFlashEffect,
        D::viewFlashADSEffect,
        D::viewBodyFlashADSEffect,
        D::signatureViewFlashEffect,
        D::signatureViewBodyFlashEffect,
        D::signatureWorldFlashEffect,
        D::signatureViewFlashADSEffect,
        D::signatureViewBodyFlashADSEffect,
        D::meleeHitEffect,
        D::meleeMissEffect,
    ] {
        fx_field(w, &mut rec, d, f)?;
    }
    for f in WEAPON_SOUNDS {
        sound_field(w, &mut rec, d, f)?;
    }
    for list in [D::bounceSound, D::rollingSound] {
        if let Some(at) = w.inline_array(d + list, 8 * SURF_TYPES, 7)? {
            for i in 0..SURF_TYPES {
                if let Some(name) = sound_custom(w, at + 8 * i)? {
                    rec.sounds.push((list, name));
                }
            }
        }
    }
    for f in [
        D::viewShellEjectEffect,
        D::worldShellEjectEffect,
        D::viewLastShotEjectEffect,
        D::worldLastShotEjectEffect,
        D::viewMagEjectEffect,
    ] {
        fx_field(w, &mut rec, d, f)?;
    }
    material_field(w, &mut rec, d, D::reticleCenter)?;
    material_field(w, &mut rec, d, D::reticleSide)?;
    rec.world_models = asset_array(w, AssetType::XModel, d + D::worldModel, 2)?;
    for f in [
        D::worldClipModel,
        D::rocketModel,
        D::knifeModel,
        D::worldKnifeModel,
    ] {
        model_field(w, &mut rec, d, f)?;
    }
    for f in [
        D::hudIcon,
        D::pickupIcon,
        D::minimapIconFriendly,
        D::minimapIconEnemy,
        D::minimapIconNeutral,
        D::ammoCounterIcon,
    ] {
        material_field(w, &mut rec, d, f)?;
    }
    for f in [D::szAmmoName, D::szClipName, D::szSharedAmmoCapName] {
        string_field(w, &mut rec, d, f)?;
    }
    w.asset(AssetType::PhysCollmap, d + D::physCollmap)?;
    w.asset(AssetType::PhysPreset, d + D::physPreset)?;
    string_field(w, &mut rec, d, D::szUseHintString)?;
    string_field(w, &mut rec, d, D::dropHintString)?;
    rec.location_damage = w
        .blob(d + D::locationDamageMultipliers, 4 * 22, 3)?
        .map(|pos| Span { pos, len: 4 * 22 });
    for f in [
        D::fireRumble,
        D::fireMedRumble,
        D::fireHighRumble,
        D::meleeImpactRumble,
    ] {
        string_field(w, &mut rec, d, f)?;
    }
    rec.tracer = w.asset(AssetType::Tracer, d + D::tracerType)?;
    w.asset(AssetType::Tracer, d + D::signatureTracerType)?;
    w.asset(AssetType::Laser, d + D::laserType)?;
    sound_field(w, &mut rec, d, D::turretOverheatSound)?;
    fx_field(w, &mut rec, d, D::turretOverheatEffect)?;
    string_field(w, &mut rec, d, D::turretBarrelSpinRumble)?;
    sound_field(w, &mut rec, d, D::turretBarrelSpinMaxSnd)?;
    for i in 0..4 {
        sound_field(w, &mut rec, d, D::turretBarrelSpinUpSnd + 8 * i)?;
    }
    for i in 0..4 {
        sound_field(w, &mut rec, d, D::turretBarrelSpinDownSnd + 8 * i)?;
    }
    sound_field(w, &mut rec, d, D::missileConeSoundAlias)?;
    sound_field(w, &mut rec, d, D::missileConeSoundAliasAtBase)?;
    model_field(w, &mut rec, d, D::stowOffsetModel)?;
    use L::TurretHydraulicSettings as T;
    if let Some(at) = w.inline_array(d + D::turretHydraulicSettings, T::SIZE, 3)? {
        for f in [
            T::verticalSound,
            T::verticalStopSound,
            T::horizontalSound,
            T::horizontalStopSound,
        ] {
            sound_custom(w, at + f)?;
        }
    }
    for f in [
        D::overlay__shader,
        D::overlay__shaderLowRes,
        D::overlay__shaderEMP,
        D::overlay__shaderEMPLowRes,
        D::killIcon,
        D::dpadIcon,
        D::hudProximityWarningIcon,
    ] {
        material_field(w, &mut rec, d, f)?;
    }
    string_field(w, &mut rec, d, D::projectileName)?;
    model_field(w, &mut rec, d, D::projectileModel)?;
    fx_field(w, &mut rec, d, D::projExplosionEffect)?;
    fx_field(w, &mut rec, d, D::projDudEffect)?;
    sound_field(w, &mut rec, d, D::projExplosionSound)?;
    sound_field(w, &mut rec, d, D::projDudSound)?;
    w.blob(d + D::parallelBounce, 4 * SURF_TYPES, 3)?;
    w.blob(d + D::perpendicularBounce, 4 * SURF_TYPES, 3)?;
    fx_field(w, &mut rec, d, D::projTrailEffect)?;
    fx_field(w, &mut rec, d, D::projBeaconEffect)?;
    fx_field(w, &mut rec, d, D::projIgnitionEffect)?;
    sound_field(w, &mut rec, d, D::projIgnitionSound)?;
    for k in 0..2 {
        string_field(w, &mut rec, d, D::accuracyGraphName + 8 * k)?;
        let n = w.s.u16_at(d + D::accuracyGraphKnotCount + 2 * k)? as usize;
        if let Some(at) = w.blob(d + D::accuracyGraphKnots + 8 * k, 8 * n, 3)? {
            rec.accuracy_graph[k] = (0..n)
                .map(|i| Ok([w.s.f32_at(at + 8 * i)?, w.s.f32_at(at + 8 * i + 4)?]))
                .collect::<Result<_>>()?;
        }
        w.blob(d + D::originalAccuracyGraphKnots + 8 * k, 8 * n, 3)?;
    }
    string_field(w, &mut rec, d, D::szScript)?;
    w.s.pop();
    w.cap.weapons.push(rec);
    Ok(AssetId {
        ty: AssetType::Weapon,
        index: (w.cap.weapons.len() - 1) as u32,
    })
}
