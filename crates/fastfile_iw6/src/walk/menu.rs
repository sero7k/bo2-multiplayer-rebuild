//! Menus (`MenuList`, `menuDef_t`): read only to stay on the stream, in the
//! order and alignment of x64-zt `iw6/assets/menudef.cpp` / `menulist.cpp`
//! (statements, event handlers and item data are VIRTUAL, aligned 4; the
//! pointer arrays aligned 8).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

#[cfg(test)]
fn trace(w: &Walker, what: &str, at: usize) {
    if let Ok(r) = std::env::var("IW6_MENU_TRACE")
        && let Some((a, b)) = r.split_once(',')
        && let (Ok(a), Ok(b)) = (
            usize::from_str_radix(a.trim_start_matches("0x"), 16),
            usize::from_str_radix(b.trim_start_matches("0x"), 16),
        )
        && (a..b).contains(&at)
    {
        eprintln!("MT {what} at {at:#x} virt {:x?}", w.vmap.virt_of(at));
    }
}
#[cfg(not(test))]
fn trace(_: &Walker, _: &str, _: usize) {}

fn statement(w: &mut Walker, field: usize) -> Result<()> {
    use L::Statement_s as S;
    let Some(s) = w.inline_array(field, S::SIZE, 3)? else {
        return Ok(());
    };
    let n = w.s.i32_at(s + S::numEntries)?.max(0) as usize;
    trace(w, "statement", s);
    let e_len = L::expressionEntry::SIZE;
    if let Some(entries) = w.inline_array(s + S::entries, e_len * n, 3)? {
        trace(w, "entries", entries);
        for i in 0..n {
            let e = entries + e_len * i;
            if w.s.i32_at(e + L::expressionEntry::r#type)? == 0 {
                continue;
            }
            match w
                .s
                .i32_at(e + L::expressionEntry::data__operand__dataType)?
            {
                2 => {
                    w.xstr(e)?;
                }
                3 => statement(w, e)?,
                _ => {}
            }
        }
    }
    supporting_data(w, s + S::supportingData)
}

fn supporting_data(w: &mut Walker, field: usize) -> Result<()> {
    use L::ExpressionSupportingData as D;
    let Some(d) = w.inline_array(field, D::SIZE, 3)? else {
        return Ok(());
    };
    trace(w, "supporting", d);
    let n = w.s.i32_at(d + D::uifunctions__totalFunctions)?.max(0) as usize;
    if let Some(arr) = w.inline_array(d + D::uifunctions__functions, 8 * n, 7)? {
        for i in 0..n {
            statement(w, arr + 8 * i)?;
        }
    }
    let n = w.s.i32_at(d + D::staticDvarList__numStaticDvars)?.max(0) as usize;
    if let Some(arr) = w.inline_array(d + D::staticDvarList__staticDvars, 8 * n, 7)? {
        for i in 0..n {
            if let Some(dv) = w.inline_array(arr + 8 * i, L::StaticDvar::SIZE, 3)? {
                w.xstr(dv + L::StaticDvar::dvarName)?;
            }
        }
    }
    let n = w.s.i32_at(d + D::uiStrings__totalStrings)?.max(0) as usize;
    if let Some(arr) = w.inline_array(d + D::uiStrings__strings, 8 * n, 7)? {
        for i in 0..n {
            w.xstr(arr + 8 * i)?;
        }
    }
    Ok(())
}

fn event_handler_set(w: &mut Walker, field: usize) -> Result<()> {
    use L::MenuEventHandlerSet as S;
    let Some(set) = w.inline_array(field, S::SIZE, 3)? else {
        return Ok(());
    };
    trace(w, "handlerset", set);
    let n = w.s.i32_at(set + S::eventHandlerCount)?.max(0) as usize;
    let Some(arr) = w.inline_array(set + S::eventHandlers, 8 * n, 7)? else {
        return Ok(());
    };
    for i in 0..n {
        let Some(h) = w.inline_array(arr + 8 * i, L::MenuEventHandler::SIZE, 3)? else {
            continue;
        };
        trace(w, "handler", h);
        let data = h + L::MenuEventHandler::eventData;
        match w.s.u8_at(h + L::MenuEventHandler::eventType)? {
            0 => {
                w.xstr(data)?;
            }
            1 => {
                if let Some(cs) = w.inline_array(data, L::ConditionalScript::SIZE, 3)? {
                    statement(w, cs + L::ConditionalScript::eventExpression)?;
                    event_handler_set(w, cs + L::ConditionalScript::eventHandlerSet)?;
                }
            }
            2 => event_handler_set(w, data)?,
            3..=6 => {
                if let Some(lv) = w.inline_array(data, L::SetLocalVarData::SIZE, 3)? {
                    w.xstr(lv + L::SetLocalVarData::localVarName)?;
                    statement(w, lv + L::SetLocalVarData::expression)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn item_key_handler(w: &mut Walker, field: usize) -> Result<()> {
    use L::ItemKeyHandler as K;
    let mut field = field;
    // (A linked list: each handler's `next` follows its action.)
    while let Some(k) = w.inline_array(field, K::SIZE, 3)? {
        trace(w, "keyhandler", k);
        event_handler_set(w, k + K::action)?;
        field = k + K::next;
    }
    Ok(())
}

fn menu_data(w: &mut Walker, field: usize) -> Result<()> {
    use L::menuData_t as M;
    let Some(m) = w.inline_array(field, M::SIZE, 3)? else {
        return Ok(());
    };
    trace(w, "menudata", m);
    supporting_data(w, m + M::expressionData)?;
    for f in [
        M::onOpen,
        M::onClose,
        M::onCloseRequest,
        M::onESC,
        M::onFocusDueToClose,
    ] {
        event_handler_set(w, m + f)?;
    }
    item_key_handler(w, m + M::onKey)?;
    statement(w, m + M::visibleExp)?;
    w.xstr(m + M::allowedBinding)?;
    w.xstr(m + M::soundName)?;
    for f in [
        M::rectXExp,
        M::rectYExp,
        M::rectWExp,
        M::rectHExp,
        M::openSoundExp,
        M::closeSoundExp,
        M::soundLoopExp,
    ] {
        statement(w, m + f)?;
    }
    Ok(())
}

/// A window's strings and background; its name.
fn window(w: &mut Walker, at: usize) -> Result<String> {
    use L::windowDef_t as W;
    let name = w.xstr(at + W::name)?.unwrap_or_default();
    w.xstr(at + W::group)?;
    w.asset(AssetType::Material, at + W::background)?;
    Ok(name)
}

fn item_type_data(w: &mut Walker, field: usize, ty: i32) -> Result<()> {
    match ty {
        6 => {
            use L::listBoxDef_s as B;
            if let Some(b) = w.inline_array(field, B::SIZE, 3)? {
                event_handler_set(w, b + B::onDoubleClick)?;
                w.asset(AssetType::Material, b + B::selectIcon)?;
                statement(w, b + B::elementHeightExp)?;
            }
        }
        0 | 4 | 9 | 10 | 11 | 14 | 16 | 17 | 18 | 22 | 23 => {
            w.inline_array(field, L::editFieldDef_s::SIZE, 3)?;
        }
        12 => {
            use L::multiDef_s as M;
            if let Some(m) = w.inline_array(field, M::SIZE, 3)? {
                for i in 0..32 {
                    w.xstr(m + M::dvarList + 8 * i)?;
                }
                for i in 0..32 {
                    w.xstr(m + M::dvarStr + 8 * i)?;
                }
            }
        }
        // (x64-zt aligns the block to 4 first; the retail zones do not:
        // `ui_mp/ingame.txt` in common_mp drifts by the difference.)
        13 => {
            w.xstr(field)?;
        }
        20 => {
            w.inline_array(field, L::newsTickerDef_s::SIZE, 3)?;
        }
        21 => {
            w.inline_array(field, L::textScrollDef_s::SIZE, 3)?;
        }
        _ => {}
    }
    Ok(())
}

fn item(w: &mut Walker, field: usize) -> Result<()> {
    use L::itemDef_t as I;
    let Some(i) = w.inline_array(field, I::SIZE, 3)? else {
        return Ok(());
    };
    trace(w, "item", i);
    window(w, i + I::window)?;
    w.xstr(i + I::text)?;
    for f in [
        I::mouseEnterText,
        I::mouseExitText,
        I::mouseEnter,
        I::mouseExit,
        I::action,
        I::accept,
        I::onFocus,
        I::hasFocus,
        I::leaveFocus,
    ] {
        event_handler_set(w, i + f)?;
    }
    w.xstr(i + I::dvar)?;
    w.xstr(i + I::dvarTest)?;
    item_key_handler(w, i + I::onKey)?;
    w.xstr(i + I::enableDvar)?;
    w.xstr(i + I::localVar)?;
    w.asset(AssetType::Sound, i + I::focusSound)?;
    let ty = w.s.i32_at(i + I::r#type)?;
    trace(w, &format!("typedata {ty}"), w.s.pos());
    item_type_data(w, i + I::typeData, ty)?;
    let n = w.s.i32_at(i + I::floatExpressionCount)?.max(0) as usize;
    let fe = L::ItemFloatExpression::SIZE;
    if let Some(arr) = w.inline_array(i + I::floatExpressions, fe * n, 3)? {
        trace(w, "floatexprs", arr);
        for k in 0..n {
            statement(w, arr + fe * k + L::ItemFloatExpression::expression)?;
        }
    }
    for f in [
        I::visibleExp,
        I::disabledExp,
        I::textExp,
        I::materialExp,
        I::textAlignYExp,
    ] {
        statement(w, i + f)?;
    }
    Ok(())
}

pub(super) fn menu(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::menuDef_t as M;
    w.s.push(VIRTUAL);
    menu_data(w, h + M::data)?;
    let name = window(w, h + M::window)?;
    let n = w.s.i32_at(h + M::itemCount)?.max(0) as usize;
    if let Some(arr) = w.inline_array(h + M::items, 8 * n, 7)? {
        for i in 0..n {
            item(w, arr + 8 * i)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::Menu, name))
}

pub(super) fn menu_list(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::MenuList as M;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + M::name)?.unwrap_or_default();
    let n = w.s.i32_at(h + M::menuCount)?.max(0) as usize;
    if let Some(arr) = w.inline_array(h + M::menus, 8 * n, 7)? {
        for i in 0..n {
            w.asset(AssetType::Menu, arr + 8 * i)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::MenuList, name))
}
