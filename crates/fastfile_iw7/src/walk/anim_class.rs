//! Animation classes: no x64-zt writer, so the order is inferred from
//! `AnimationClass` (x64-zt `iw7/structs.hpp`) and checked against the
//! retail data (`iw7_common_core_mp.ff`'s one anim class): the class
//! name, the state machine (header, its states, then per state: anim
//! entries, the anim indices (RUNTIME: filled at load, no file data),
//! the alias list, then each alias's infos; then the aim sets), the
//! scriptable, the sound arrays (`scr_string_t`s), the effect notes, the
//! effects (`FxCombinedDef`s, then each effect), the effect tags.
//!
//! Aim sets do not occur in the sample; their order (anim names in
//! VIRTUAL, the index arrays in RUNTIME like the states') is a guess.

use super::weapon::fx_combined;
use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::{RUNTIME, VIRTUAL};

/// A pointer to data the loader allocates in RUNTIME (no file data).
fn runtime(w: &mut Walker, field: usize, len: usize) -> Result<()> {
    if w.s.ptr_at(field)?.is_inline() {
        w.s.push(RUNTIME);
        w.inline_array(field, len, 7)?;
        w.s.pop();
    }
    Ok(())
}

fn state_machine(w: &mut Walker, field: usize) -> Result<()> {
    use L::AnimAlias as A;
    use L::AnimationAimSet as M;
    use L::AnimationState as St;
    use L::AnimationStateMachine as S;
    let Some(sm) = w.inline_array(field, S::SIZE, 7)? else {
        return Ok(());
    };
    let states = w.s.u16_at(sm + S::stateCount)? as usize;
    let aims = w.s.u16_at(sm + S::aimSetCount)? as usize;
    if let Some(t) = w.inline_array(sm + S::states, St::SIZE * states, 7)? {
        for i in 0..states {
            let st = t + St::SIZE * i;
            let entries = w.s.u8_at(st + St::entryCount)? as usize;
            let aliases = w.s.u8_at(st + St::aliasCount)? as usize;
            w.blob(st + St::animEntries, L::AnimationEntry::SIZE * entries, 3)?;
            runtime(w, st + St::animIndices, 8 * entries)?;
            if let Some(al) = w.inline_array(st + St::aliasList, A::SIZE * aliases, 7)? {
                for k in 0..aliases {
                    let a = al + A::SIZE * k;
                    let n = w.s.u8_at(a + A::animCount)? as usize;
                    w.blob(a + A::aliasInfo, L::AnimAliasInfo::SIZE * n, 3)?;
                }
            }
        }
    }
    if let Some(t) = w.inline_array(sm + S::aimSets, M::SIZE * aims, 7)? {
        for i in 0..aims {
            let m = t + M::SIZE * i;
            let n = w.s.u32_at(m + M::animCount)? as usize;
            w.blob(m + M::animName, 4 * n, 3)?;
            runtime(w, m + M::animIndices, 8 * n)?;
            runtime(w, m + M::aimNodeIndices, 8 * n)?;
        }
    }
    Ok(())
}

pub(super) fn anim_class(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::AnimationClass as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::className)?.unwrap_or_default();
    state_machine(w, h + C::stateMachine)?;
    w.asset(AssetType::Scriptable, h + C::scriptable)?;
    let sounds = w.s.u16_at(h + C::soundCount)? as usize;
    let effects = w.s.u16_at(h + C::effectCount)? as usize;
    for f in [C::soundNotes, C::soundNames, C::soundOptions] {
        w.blob(h + f, 4 * sounds, 3)?;
    }
    w.blob(h + C::effectNotes, 4 * effects, 3)?;
    if let Some(t) = w.inline_array(h + C::effectDefs, L::FxCombinedDef::SIZE * effects, 7)? {
        for i in 0..effects {
            fx_combined(w, t + L::FxCombinedDef::SIZE * i)?;
        }
    }
    w.blob(h + C::effectTags, 4 * effects, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::AnimClass, name))
}
