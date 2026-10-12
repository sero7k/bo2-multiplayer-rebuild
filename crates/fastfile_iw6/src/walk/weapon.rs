//! Weapons (`WeaponCompleteDef` + `WeaponDef`) and attachments.

use super::misc::sound_custom;
use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{AnimOverride, AttachmentRec, Span, WeaponRec};

/// Weapon animations (`NUM_WEAP_ANIMS`).
pub const NUM_WEAP_ANIMS: usize = 124;

/// An array of `n` asset pointers (VIRTUAL, aligned 8).
fn asset_array(w: &mut Walker, ty: AssetType, field: usize, n: usize) -> Result<Vec<Option<AssetId>>> {
    let mut out = Vec::new();
    if let Some(at) = w.inline_array(field, 8 * n, 7)? {
        for i in 0..n {
            out.push(w.asset(ty, at + 8 * i)?);
        }
    } else if let Some(v) = w.s.ptr_at(field)?.virtual_offset()
        && let Some(at) = w.vmap.file_pos(v)
    {
        for i in 0..n {
            out.push(w.s.ptr_at(at + 8 * i)?.virtual_offset().and_then(|v| w.resolve_slot(ty, v)));
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
        record: Span { pos: h, len: A::SIZE },
        ..Default::default()
    };
    rec.world_models = asset_array(w, AssetType::XModel, h + A::worldModels, 64)?;
    rec.view_models = asset_array(w, AssetType::XModel, h + A::viewModels, 64)?;
    rec.reticle_view_models = asset_array(w, AssetType::XModel, h + A::reticleViewModels, 16)?;
    let parts: [(&'static str, usize, usize, u64); 22] = [
        ("ammogeneral", A::ammogeneral, L::AttAmmoGeneral::SIZE, 3),
        ("sight", A::sight, L::AttSight::SIZE, 3),
        ("reload", A::reload, L::AttReload::SIZE, 1),
        ("addOns", A::addOns, L::AttAddOns::SIZE, 0),
        ("general", A::general, L::AttGeneral::SIZE, 3),
        ("aimAssist", A::aimAssist, L::AttAimAssist::SIZE, 3),
        ("ammunition", A::ammunition, L::AttAmmunition::SIZE, 3),
        ("damage", A::damage, L::AttDamage::SIZE, 3),
        ("locationDamage", A::locationDamage, L::AttLocationDamage::SIZE, 3),
        ("idleSettings", A::idleSettings, L::AttIdleSettings::SIZE, 3),
        ("adsSettings", A::adsSettings, L::AttADSSettings::SIZE, 3),
        ("adsSettingsMain", A::adsSettingsMain, L::AttADSSettings::SIZE, 3),
        ("scopeDriftSettings", A::scopeDriftSettings, L::AttScopeDriftSettings::SIZE, 3),
        ("scopeDriftSettingsMain", A::scopeDriftSettingsMain, L::AttScopeDriftSettings::SIZE, 3),
        ("hipSpread", A::hipSpread, L::AttHipSpread::SIZE, 3),
        ("gunKick", A::gunKick, L::AttGunKick::SIZE, 3),
        ("viewKick", A::viewKick, L::AttViewKick::SIZE, 3),
        ("adsOverlay", A::adsOverlay, L::AttADSOverlay::SIZE, 3),
        ("ui", A::ui, L::AttUI::SIZE, 3),
        ("rumbles", A::rumbles, L::AttRumbles::SIZE, 3),
        ("projectile", A::projectile, L::AttProjectile::SIZE, 3),
        ("", 0, 0, 0),
    ];
    for (part, field, len, mask) in parts {
        if part.is_empty() {
            continue;
        }
        let Some(at) = w.inline_array(h + field, len, mask)? else { continue };
        rec.parts.push((part, Span { pos: at, len }));
        match part {
            "ammogeneral" => {
                w.asset(AssetType::Tracer, at + L::AttAmmoGeneral::tracerType)?;
            }
            "general" => {
                w.asset(AssetType::Material, at + L::AttGeneral::reticleCenter)?;
                w.asset(AssetType::Material, at + L::AttGeneral::reticleSide)?;
            }
            "adsOverlay" => {
                use L::AttADSOverlay as O;
                for f in [O::overlay__shader, O::overlay__shaderLowRes, O::overlay__shaderEMP, O::overlay__shaderEMPLowRes] {
                    w.asset(AssetType::Material, at + f)?;
                }
            }
            "ui" => {
                w.asset(AssetType::Material, at + L::AttUI::dpadIcon)?;
                w.asset(AssetType::Material, at + L::AttUI::ammoCounterIcon)?;
            }
            "rumbles" => {
                w.xstr(at + L::AttRumbles::fireRumble)?;
                w.xstr(at + L::AttRumbles::meleeImpactRumble)?;
            }
            "projectile" => {
                use L::AttProjectile as P;
                w.xstr(at + P::projectileName)?;
                w.asset(AssetType::XModel, at + P::projectileModel)?;
                w.asset(AssetType::Fx, at + P::projExplosionEffect)?;
                sound_custom(w, at + P::projExplosionSound)?;
                w.asset(AssetType::Fx, at + P::projDudEffect)?;
                sound_custom(w, at + P::projDudSound)?;
                w.asset(AssetType::Fx, at + P::projTrailEffect)?;
                w.asset(AssetType::Fx, at + P::projIgnitionEffect)?;
                sound_custom(w, at + P::projIgnitionSound)?;
            }
            _ => {}
        }
    }
    w.s.pop();
    w.cap.attachments.push(rec);
    Ok(AssetId { ty: AssetType::Attachment, index: (w.cap.attachments.len() - 1) as u32 })
}

const WEAPON_SOUNDS: [usize; 49] = {
    use L::WeaponDef as D;
    [
        D::pickupSound,
        D::pickupSoundPlayer,
        D::ammoPickupSound,
        D::ammoPickupSoundPlayer,
        D::projectileSound,
        D::pullbackSound,
        D::pullbackSoundPlayer,
        D::fireSound,
        D::fireSoundPlayer,
        D::fireSoundPlayerAkimbo,
        D::fireLoopSound,
        D::fireLoopSoundPlayer,
        D::fireStopSound,
        D::fireStopSoundPlayer,
        D::fireLastSound,
        D::fireLastSoundPlayer,
        D::emptyFireSound,
        D::emptyFireSoundPlayer,
        D::meleeSwipeSound,
        D::meleeSwipeSoundPlayer,
        D::meleeHitSound,
        D::meleeMissSound,
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
        usize::MAX,
    ]
};

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

/// The `WeaponDef` (VIRTUAL, aligned 4) and everything it points at.
fn weapon_def(w: &mut Walker, rec: &mut WeaponRec, field: usize) -> Result<()> {
    use L::WeaponDef as D;
    let Some(d) = w.inline_array(field, D::SIZE, 3)? else { return Ok(()) };
    rec.def = Some(Span { pos: d, len: D::SIZE });
    string_field(w, rec, d, D::szOverlayName)?;
    rec.gun_models = asset_array(w, AssetType::XModel, d + D::gunXModel, 64)?;
    rec.hand_model = w.asset(AssetType::XModel, d + D::handXModel)?;
    model_field(w, rec, d, D::camoWorldModel)?;
    model_field(w, rec, d, D::camoViewModel)?;
    material_field(w, rec, d, D::camoWorldModelMaterialOverride)?;
    material_field(w, rec, d, D::camoViewModelMaterialOverride)?;
    asset_array(w, AssetType::Material, d + D::camoMaterialTarget, 63)?;
    rec.anims_right = asset_array(w, AssetType::XAnimParts, d + D::szXAnimsRightHanded, NUM_WEAP_ANIMS)?;
    rec.anims_left = asset_array(w, AssetType::XAnimParts, d + D::szXAnimsLeftHanded, NUM_WEAP_ANIMS)?;
    string_field(w, rec, d, D::szModeName)?;
    rec.notetrack_sound_keys = script_strings(w, d + D::notetrackSoundMapKeys, 24)?;
    rec.notetrack_sound_values = script_strings(w, d + D::notetrackSoundMapValues, 24)?;
    script_strings(w, d + D::notetrackRumbleMapKeys, 16)?;
    script_strings(w, d + D::notetrackRumbleMapValues, 16)?;
    script_strings(w, d + D::notetrackFXMapKeys, 16)?;
    asset_array(w, AssetType::Fx, d + D::notetrackFXMapValues, 16)?;
    script_strings(w, d + D::notetrackFXMapTagValues, 16)?;
    for f in [D::viewFlashEffect, D::worldFlashEffect, D::viewFlashADSEffect] {
        fx_field(w, rec, d, f)?;
    }
    for f in WEAPON_SOUNDS.iter().copied().filter(|&f| f != usize::MAX) {
        sound_field(w, rec, d, f)?;
    }
    for list in [D::bounceSound, D::rollingSound] {
        if let Some(at) = w.inline_array(d + list, 8 * 31, 7)? {
            for i in 0..31 {
                if let Some(name) = sound_custom(w, at + 8 * i)? {
                    rec.sounds.push((list, name));
                }
            }
        }
    }
    for f in [D::viewShellEjectEffect, D::worldShellEjectEffect, D::viewLastShotEjectEffect, D::worldLastShotEjectEffect, D::viewMagEjectEffect] {
        fx_field(w, rec, d, f)?;
    }
    material_field(w, rec, d, D::reticleCenter)?;
    material_field(w, rec, d, D::reticleSide)?;
    rec.world_models = asset_array(w, AssetType::XModel, d + D::worldModel, 64)?;
    for f in [D::worldClipModel, D::rocketModel, D::knifeModel, D::worldKnifeModel] {
        model_field(w, rec, d, f)?;
    }
    for f in [D::hudIcon, D::pickupIcon, D::ammoCounterIcon] {
        material_field(w, rec, d, f)?;
    }
    for f in [D::szAmmoName, D::szClipName, D::szSharedAmmoCapName] {
        string_field(w, rec, d, f)?;
    }
    for f in [D::overlay__shader, D::overlay__shaderLowRes, D::overlay__shaderEMP, D::overlay__shaderEMPLowRes] {
        material_field(w, rec, d, f)?;
    }
    w.asset(AssetType::PhysCollmap, d + D::physCollmap)?;
    string_field(w, rec, d, D::projectileName)?;
    model_field(w, rec, d, D::projectileModel)?;
    fx_field(w, rec, d, D::projExplosionEffect)?;
    fx_field(w, rec, d, D::projDudEffect)?;
    sound_field(w, rec, d, D::projExplosionSound)?;
    sound_field(w, rec, d, D::projDudSound)?;
    // (The zone tools null these two; the retail data has them: a float per
    // surface type.)
    w.blob(d + D::parallelBounce, 4 * 31, 3)?;
    w.blob(d + D::perpendicularBounce, 4 * 31, 3)?;
    fx_field(w, rec, d, D::projTrailEffect)?;
    fx_field(w, rec, d, D::projBeaconEffect)?;
    fx_field(w, rec, d, D::projIgnitionEffect)?;
    sound_field(w, rec, d, D::projIgnitionSound)?;
    // The accuracy graphs' names and original knots (also nulled by the
    // tools).
    for k in 0..2 {
        string_field(w, rec, d, D::accuracyGraphName + 8 * k)?;
        let n = w.s.u16_at(d + D::originalAccuracyGraphKnotCount + 2 * k)? as usize;
        if let Some(at) = w.blob(d + D::originalAccuracyGraphKnots + 8 * k, 8 * n, 3)? {
            rec.accuracy_graph[k] = (0..n).map(|i| Ok([w.s.f32_at(at + 8 * i)?, w.s.f32_at(at + 8 * i + 4)?])).collect::<Result<_>>()?;
        }
    }
    for f in [D::szUseHintString, D::dropHintString, D::szScript] {
        string_field(w, rec, d, f)?;
    }
    rec.location_damage = w.blob(d + D::locationDamageMultipliers, 4 * 22, 3)?.map(|pos| Span { pos, len: 4 * 22 });
    string_field(w, rec, d, D::fireRumble)?;
    string_field(w, rec, d, D::meleeImpactRumble)?;
    rec.tracer = w.asset(AssetType::Tracer, d + D::tracerType)?;
    sound_field(w, rec, d, D::turretOverheatSound)?;
    fx_field(w, rec, d, D::turretOverheatEffect)?;
    string_field(w, rec, d, D::turretBarrelSpinRumble)?;
    sound_field(w, rec, d, D::turretBarrelSpinMaxSnd)?;
    for i in 0..4 {
        sound_field(w, rec, d, D::turretBarrelSpinUpSnd + 8 * i)?;
        sound_field(w, rec, d, D::turretBarrelSpinDownSnd + 8 * i)?;
    }
    sound_field(w, rec, d, D::missileConeSoundAlias)?;
    sound_field(w, rec, d, D::missileConeSoundAliasAtBase)?;
    model_field(w, rec, d, D::stowOffsetModel)?;
    Ok(())
}

pub(super) fn weapon(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::WeaponCompleteDef as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::szInternalName)?.unwrap_or_default();
    let mut rec = WeaponRec { name, complete: Span { pos: h, len: C::SIZE }, ..Default::default() };
    weapon_def(w, &mut rec, h + C::weapDef)?;
    rec.display_name = w.xstr(h + C::szDisplayName)?.unwrap_or_default();
    rec.hide_tags = script_strings(w, h + C::hideTags, 32)?;
    rec.scopes = asset_array(w, AssetType::Attachment, h + C::scopes, 6)?;
    rec.under_barrels = asset_array(w, AssetType::Attachment, h + C::underBarrels, 3)?;
    rec.others = asset_array(w, AssetType::Attachment, h + C::others, 8)?;
    rec.anims = asset_array(w, AssetType::XAnimParts, h + C::szXAnims, NUM_WEAP_ANIMS)?;
    let n = w.s.u32_at(h + C::numAnimOverrides)? as usize;
    let ao = L::AnimOverrideEntry::SIZE;
    if let Some(at) = w.inline_array(h + C::animOverrides, ao * n, 3)? {
        use L::AnimOverrideEntry as E;
        for i in 0..n {
            let e = at + ao * i;
            let override_anim = w.asset(AssetType::XAnimParts, e + E::overrideAnim)?;
            let altmode_anim = w.asset(AssetType::XAnimParts, e + E::altmodeAnim)?;
            rec.anim_overrides.push(AnimOverride {
                attachment1: w.s.u16_at(e + E::attachment1)? as u32,
                attachment2: w.s.u16_at(e + E::attachment2)? as u32,
                anim_index: w.s.u32_at(e + E::animTreeType)?,
                override_anim,
                altmode_anim,
                anim_time: w.s.i32_at(e + E::animTime)?,
                altmode_time: w.s.i32_at(e + E::altTime)?,
            });
        }
    }
    let n = w.s.u32_at(h + C::numSoundOverrides)? as usize;
    let so = L::SoundOverrideEntry::SIZE;
    if let Some(at) = w.inline_array(h + C::soundOverrides, so * n, 3)? {
        use L::SoundOverrideEntry as E;
        for i in 0..n {
            let e = at + so * i;
            let a = sound_custom(w, e + E::overrideSound)?;
            let b = sound_custom(w, e + E::altmodeSound)?;
            rec.sound_overrides.push((w.s.u16_at(e + E::attachment1)? as u32, w.s.u16_at(e + E::attachment2)? as u32, a, b));
        }
    }
    let n = w.s.u32_at(h + C::numFXOverrides)? as usize;
    let fo = L::FXOverrideEntry::SIZE;
    if let Some(at) = w.inline_array(h + C::fxOverrides, fo * n, 3)? {
        for i in 0..n {
            let e = at + fo * i;
            w.asset(AssetType::Fx, e + L::FXOverrideEntry::overrideFX)?;
            w.asset(AssetType::Fx, e + L::FXOverrideEntry::altmodeFX)?;
        }
    }
    let n = w.s.u32_at(h + C::numReloadStateTimerOverrides)? as usize;
    w.blob(h + C::reloadOverrides, L::ReloadStateTimerEntry::SIZE * n, 3)?;
    let n = w.s.u32_at(h + C::numNotetrackOverrides)? as usize;
    let no = L::NoteTrackToSoundEntry::SIZE;
    if let Some(at) = w.inline_array(h + C::notetrackOverrides, no * n, 3)? {
        for i in 0..n {
            let e = at + no * i;
            w.blob(e + L::NoteTrackToSoundEntry::notetrackSoundMapKeys, 4 * 24, 3)?;
            w.blob(e + L::NoteTrackToSoundEntry::notetrackSoundMapValues, 4 * 24, 3)?;
        }
    }
    rec.alt_weapon_name = w.xstr(h + C::szAltWeaponName)?.unwrap_or_default();
    if let Some(id) = w.asset(AssetType::Material, h + C::killIcon)? {
        rec.materials.push((usize::MAX - C::killIcon, id));
    }
    if let Some(id) = w.asset(AssetType::Material, h + C::dpadIcon)? {
        rec.materials.push((usize::MAX - C::dpadIcon, id));
    }
    for k in 0..2 {
        let n = w.s.u16_at(h + C::accuracyGraphKnotCount + 2 * k)? as usize;
        w.blob(h + C::accuracyGraphKnots + 8 * k, 8 * n, 3)?;
    }
    w.xstr(h + C::szAdsrBaseSetting)?;
    w.s.pop();
    w.cap.weapons.push(rec);
    Ok(AssetId { ty: AssetType::Weapon, index: (w.cap.weapons.len() - 1) as u32 })
}
