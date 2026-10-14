//! Scriptables (x64-zt `iw7/assets/scriptabledef.cpp`): the def, its parts
//! (recursive: parts own states own events, events own events and part
//! references), and its models, in the writer's order.

use super::weapon::{asset_array, fx_combined};
use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};
use crate::layout as L;
use crate::stream::VIRTUAL;

/// A pointer to a `{ name, flags }` base (16 bytes, aligned 8) and its name.
fn base_def(w: &mut Walker, field: usize) -> Result<()> {
    if let Some(b) = w.inline_array(field, L::ScriptableEventBaseDef::SIZE, 7)? {
        w.xstr(b + L::ScriptableEventBaseDef::name)?;
    }
    Ok(())
}

/// A `ScriptablePartReference` stored in place at `r`.
fn part_reference(w: &mut Walker, r: usize) -> Result<()> {
    use L::ScriptablePartReference as R;
    w.xstr(r + R::base__name)?;
    let ty = w.s.u32_at(r + R::r#type)?;
    if ty <= 3 {
        base_def(w, r + R::u____0__base)?;
    }
    if ty == 3 {
        let n = w.s.u32_at(r + R::u____3__count)? as usize;
        w.blob(r + R::u____3__val, 4 * n, 3)?;
    }
    Ok(())
}

/// A pointer to `n` events (aligned 8), each walked in turn.
fn events(w: &mut Walker, field: usize, n: usize) -> Result<()> {
    let len = L::ScriptableEventDef::SIZE;
    if let Some(table) = w.inline_array(field, len * n, 7)? {
        for i in 0..n {
            event(w, table + len * i)?;
        }
    }
    Ok(())
}

fn event(w: &mut Walker, e: usize) -> Result<()> {
    use AssetType as T;
    let name = w.xstr(e + L::ScriptableEventDef::base__name)?;
    let ty = w.s.u32_at(e + L::ScriptableEventDef::r#type)?;
    if std::env::var_os("IW7_SDUMP").is_some() {
        eprintln!("  event {name:?} type {ty:#x} at {e:#x} pos {:#x}", w.s.pos());
    }
    let d = e + L::ScriptableEventDef::data;
    if ty >= 0x1D {
        return Err(ReadError::Invalid("scriptable event type", e));
    }
    // Every event starts with its base pointer.
    base_def(w, d)?;
    let count = |w: &Walker, f: usize| w.s.u32_at(d + f).map(|n| n as usize);
    match ty {
        // StateChange, ChunkDynent, PartDamage: the reference is at +8.
        0x0 | 0x9 | 0x12 => part_reference(w, d + L::ScriptableEventStateChangeDef::partReference)?,
        0x2 => {
            use L::ScriptableEventRandomDef as D;
            let n = count(w, D::eventACount)?;
            events(w, d + D::eventsA, n)?;
            let n = count(w, D::eventBCount)?;
            events(w, d + D::eventsB, n)?;
        }
        0x3 => {
            w.xstr(d + L::ScriptableEventScriptDef::notification)?;
        }
        0x4 => {
            w.asset(T::XModel, d + L::ScriptableEventModelDef::model)?;
        }
        0x5 => {
            w.xstr(d + L::ScriptableEventCollisionDef::collmapName)?;
        }
        0x6 => {
            use L::ScriptableEventAnimationDef as D;
            w.asset(T::XAnimParts, d + D::animation)?;
            let n = count(w, D::eventAtEndCount)?;
            events(w, d + D::eventsAtEnd, n)?;
        }
        0x7 => {
            w.xstr(d + L::ScriptableEventHideShowBoneDef::tagName)?;
        }
        0x8 => {
            use L::ScriptableEventNoteTrackDef as D;
            use L::ScriptableNoteTrackDef as N;
            let n = count(w, D::noteTrackCount)?;
            if let Some(table) = w.inline_array(d + D::noteTracks, N::SIZE * n, 7)? {
                for i in 0..n {
                    let t = table + N::SIZE * i;
                    w.xstr(t + N::noteTrackName)?;
                    let m = w.s.u32_at(t + N::numEvents)? as usize;
                    events(w, t + N::events, m)?;
                }
            }
        }
        0xA => {
            use L::ScriptableEventSpawnDynentDef as D;
            w.asset(T::XModel, d + D::model)?;
            w.xstr(d + D::tagName)?;
        }
        0xB => {
            use L::ScriptableEventPFXDef as D;
            let n = count(w, D::scrTagCount)?;
            w.blob(d + D::scrTagNames, 4 * n, 3)?;
            let n = count(w, D::scrEndTagCount)?;
            w.blob(d + D::scrEndTagNames, 4 * n, 3)?;
            w.xstr(d + D::effectAlias)?;
            fx_combined(w, d + D::effectDef)?;
        }
        0xC => {
            use L::ScriptableEventSoundDef as D;
            w.xstr(d + D::tagName)?;
            w.xstr(d + D::soundAlias)?;
            w.xstr(d + D::soundAliasCache)?;
        }
        0xD => {
            use L::ScriptableEventExplosionDef as D;
            // x64-zt writes nothing for the weapon pointer; load it if set.
            w.asset(T::Weapon, d + D::weapon)?;
            w.xstr(d + D::tagName)?;
        }
        0xE => {
            w.xstr(d + L::ScriptableEventLightDef::name)?;
        }
        0x10 => {
            use L::ScriptableEventRumbleDef as D;
            w.xstr(d + D::tagName)?;
            w.xstr(d + D::rumble)?;
            w.asset(T::Rumble, d + D::rumbleAsset)?;
        }
        0x11 => {
            w.xstr(d + L::ScriptableEventScreenshakeDef::tagName)?;
        }
        0x13 => {
            w.asset(T::Mayhem, d + L::ScriptableEventSetMayhemDef::mayhem)?;
        }
        0x17 => {
            use L::ScriptableEventClientViewSelectorDef as D;
            let n = count(w, D::event1pCount)?;
            events(w, d + D::events1p, n)?;
            let n = count(w, D::event3pCount)?;
            events(w, d + D::events3p, n)?;
        }
        0x18 => {
            use L::ScriptableEventTeamSelectorDef as D;
            let n = count(w, D::eventPassCount)?;
            events(w, d + D::eventsPass, n)?;
            let n = count(w, D::eventFailCount)?;
            events(w, d + D::eventsFail, n)?;
        }
        0x19 => {
            use L::ScriptableEventAddModelDef as D;
            w.xstr(d + D::tagName)?;
            w.asset(T::XModel, d + D::model)?;
        }
        0x1B => {
            use L::ScriptableEventCompassIconDef as D;
            for f in (D::friendlyArrow..=D::enemyCompassIconDirectional).step_by(8) {
                w.asset(T::Material, d + f)?;
            }
        }
        0x1C => {
            w.asset(T::Material, d + L::ScriptableEventMaterialOverrideDef::material)?;
        }
        // Wait, Sun, PlayMayhem, ViewmodelShaderParam,
        // ViewmodelChangeImage, ApplyForce: the base only.
        _ => {}
    }
    Ok(())
}

/// A `ScriptableStateBaseDef` at `b`: its name and events.
fn state_base(w: &mut Walker, b: usize) -> Result<()> {
    use L::ScriptableStateBaseDef as B;
    let name = w.xstr(b + B::name)?;
    if std::env::var_os("IW7_SDUMP").is_some() {
        eprintln!("  state {name:?} events {} at {b:#x}", w.s.u32_at(b + B::numEvents)?);
    }
    let n = w.s.u32_at(b + B::numEvents)? as usize;
    events(w, b + B::events, n)
}

/// A pointer to a separate state base (aligned 8).
fn state_base_ptr(w: &mut Walker, field: usize) -> Result<()> {
    if let Some(b) = w.inline_array(field, L::ScriptableStateBaseDef::SIZE, 7)? {
        state_base(w, b)?;
    }
    Ok(())
}

fn state(w: &mut Walker, s: usize) -> Result<()> {
    use L::ScriptableStateDef as S;
    state_base(w, s + S::base)?;
    let d = s + S::data;
    match w.s.u32_at(s + S::r#type)? {
        0 => state_base_ptr(w, d + L::ScriptableStateSimpleDef::base)?,
        1 => {
            state_base_ptr(w, d + L::ScriptableStateHealthDef::base)?;
            w.xstr(d + L::ScriptableStateHealthDef::script_id)?;
        }
        2 => {
            state_base_ptr(w, d + L::ScriptableStateScriptedDef::base)?;
            w.xstr(d + L::ScriptableStateScriptedDef::script_id)?;
        }
        _ => {}
    }
    Ok(())
}

fn part(w: &mut Walker, p: usize) -> Result<()> {
    use L::ScriptablePartDef as P;
    let name = w.xstr(p + P::name)?;
    let tag = w.xstr(p + P::tagName)?;
    if std::env::var_os("IW7_SDUMP").is_some() {
        eprintln!(" part {name:?} tag {tag:?} states {} children {} at {p:#x}", w.s.u32_at(p + P::numStates)?, w.s.u32_at(p + P::numChildParts)?);
    }
    let n = w.s.u32_at(p + P::numStates)? as usize;
    let len = L::ScriptableStateDef::SIZE;
    if let Some(table) = w.inline_array(p + P::states, len * n, 7)? {
        for i in 0..n {
            state(w, table + len * i)?;
        }
    }
    let n = w.s.u32_at(p + P::numChildParts)? as usize;
    parts(w, p + P::childParts, n)?;
    let n = w.s.u32_at(p + P::numDamageTagOverrides)? as usize;
    let len = L::ScriptableDamageTagOverride::SIZE;
    if let Some(table) = w.inline_array(p + P::damageTagOverrides, len * n, 7)? {
        for i in 0..n {
            w.xstr(table + len * i + L::ScriptableDamageTagOverride::tag)?;
        }
    }
    Ok(())
}

fn parts(w: &mut Walker, field: usize, n: usize) -> Result<()> {
    let len = L::ScriptablePartDef::SIZE;
    if let Some(table) = w.inline_array(field, len * n, 7)? {
        for i in 0..n {
            part(w, table + len * i)?;
        }
    }
    Ok(())
}

pub(super) fn scriptable(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ScriptableDef as D;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + D::name)?.unwrap_or_default();
    let dbg = std::env::var_os("IW7_SDUMP").is_some();
    if dbg {
        eprintln!("scriptable {name} header {h:#x} pos {:#x} virt {:#x}", w.s.pos(), w.s.offset(VIRTUAL));
    }
    w.asset(AssetType::Scriptable, h + D::nextScriptableDef)?;
    let n = w.s.u32_at(h + D::numParts)? as usize;
    parts(w, h + D::parts, n)?;
    let n = w.s.u32_at(h + D::numXModels)? as usize;
    asset_array(w, AssetType::XModel, h + D::models, n)?;
    if dbg {
        eprintln!("scriptable {name} done pos {:#x} virt {:#x}", w.s.pos(), w.s.offset(VIRTUAL));
    }
    w.s.pop();
    Ok(w.name_only(AssetType::Scriptable, name))
}
