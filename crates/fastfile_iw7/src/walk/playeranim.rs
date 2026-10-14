//! Player anim scripts: no x64-zt writer, so the order is inferred from
//! `PlayerAnimScript` (x64-zt `iw7/structs.hpp`) and checked against the
//! retail data: each pointer in declaration order, arrays of structs with
//! pointers read whole and then each element's pointers.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

/// Fixed lengths of `scriptAnims` / `scriptEvents`.
const SCRIPT_ANIMS: usize = 121;
const SCRIPT_EVENTS: usize = 55;

fn dbg() -> bool {
    std::env::var_os("IW7_PADUMP").is_some()
}

/// `n` `PlayerAnimScriptEntry`s, then each one's items and transitions.
fn entries(w: &mut Walker, field: usize, n: usize) -> Result<()> {
    use L::PlayerAnimScriptEntry as E;
    if let Some(t) = w.inline_array(field, E::SIZE * n, 7)? {
        for i in 0..n {
            let e = t + E::SIZE * i;
            let items = w.s.u32_at(e + E::itemCount)? as usize;
            let trans = w.s.u32_at(e + E::transitionCount)? as usize;
            w.blob(e + E::items, 4 * items, 3)?;
            w.blob(e + E::transitions, 4 * trans, 3)?;
        }
    }
    Ok(())
}

pub(super) fn playeranim(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::PlayerAnimScript as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::scriptName)?.unwrap_or_default();
    if dbg() {
        eprintln!("playeranim {name} header {h:#x} pos {:#x}", w.s.pos());
        for o in (0..S::SIZE).step_by(8) {
            eprintln!("  +{o:3} {:#018x}", w.s.u64_at(h + o)?);
        }
    }
    let count = |w: &Walker, f: usize| w.s.u32_at(h + f).map(|n| n as usize);
    let n = count(w, S::animationCount)?;
    w.blob(h + S::animations, L::PlayerAnimEntry::SIZE * n, 3)?;
    if dbg() {
        eprintln!("  after animations pos {:#x}", w.s.pos());
    }
    entries(w, h + S::scriptAnims, SCRIPT_ANIMS)?;
    entries(w, h + S::scriptEvents, SCRIPT_EVENTS)?;
    if dbg() {
        eprintln!("  after entries pos {:#x}", w.s.pos());
    }
    let n = count(w, S::scriptItemCount)?;
    {
        use L::PlayerAnimScriptItem as I;
        if let Some(t) = w.inline_array(h + S::scriptItems, I::SIZE * n, 7)? {
            for i in 0..n {
                let e = t + I::SIZE * i;
                let c = w.s.u32_at(e + I::conditionCount)? as usize;
                let m = w.s.u32_at(e + I::commandCount)? as usize;
                w.blob(e + I::conditions, L::PlayerAnimScriptCondition::SIZE * c, 3)?;
                w.blob(e + I::commands, L::PlayerAnimScriptCommand::SIZE * m, 1)?;
            }
        }
    }
    if dbg() {
        eprintln!("  after items pos {:#x}", w.s.pos());
    }
    let n = count(w, S::scriptTransitionCount)?;
    w.blob(h + S::scriptTransitions, 12 * n, 3)?;
    let n = count(w, S::scriptIdleTurnCount)?;
    w.blob(h + S::scriptIdleTurns, L::PlayerAnimScriptIdleTurn::SIZE * n, 1)?;
    let n = count(w, S::scriptIdleTwitchCount)?;
    {
        use L::PlayerAnimScriptIdleTwitch as T;
        if let Some(t) = w.inline_array(h + S::scriptIdleTwitches, T::SIZE * n, 7)? {
            for i in 0..n {
                let e = t + T::SIZE * i;
                let c = w.s.u32_at(e + T::twitchCount)? as usize;
                w.blob(e + T::twitches, 2 * c, 1)?;
            }
        }
    }
    let n = count(w, S::scriptAimSetCount)?;
    w.blob(h + S::scriptAimSets, L::PlayerAnimScriptAimSet::SIZE * n, 1)?;
    let n = count(w, S::scriptLeanSetCount)?;
    w.blob(h + S::scriptLeanSets, L::PlayerAnimScriptLeanSet::SIZE * n, 1)?;
    let n = count(w, S::unkCount)?;
    w.blob(h + S::unk, 48 * n, 3)?;
    if dbg() {
        eprintln!("  before maps pos {:#x}", w.s.pos());
    }
    let anims = count(w, S::animationCount)?;
    let torso = count(w, S::torsoAnimCount)?;
    let legs = count(w, S::legsAnimCount)?;
    w.blob(h + S::torsoAnimPackMap, 2 * anims, 1)?;
    w.blob(h + S::torsoAnimUnpackMap, 2 * torso, 1)?;
    w.blob(h + S::legsAnimPackMap, 2 * anims, 1)?;
    w.blob(h + S::legsAnimUnpackMap, 2 * legs, 1)?;
    if dbg() {
        eprintln!("  before xanims pos {:#x}", w.s.pos());
    }
    let n = count(w, S::xAnimCount)?;
    super::weapon::asset_array(w, AssetType::XAnimParts, h + S::xAnims, n)?;
    if dbg() {
        eprintln!("  end pos {:#x}", w.s.pos());
    }
    w.s.pop();
    Ok(w.name_only(AssetType::PlayerAnim, name))
}
