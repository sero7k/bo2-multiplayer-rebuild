//! bo2zm: the "Omni Movement" row in BO2's Controls menu (off
//! by default, an option in the Controls menu for whoever wants it).
//!
//! Not one of Black Ops II's rows (BO2 sprints forward only), so it goes
//! last on the Move tab, after STRAFE, built the way BO2 builds the Look
//! tab's rows: a half-row spacer (`CoD.CoD9Button.Height / 2`), a
//! hardware-profile left/right selector, and Off/On by BO2's own
//! `CoD.Options.Button_AddChoices_OnOrOff` (OFF 0, ON 1).

use hks_t6::host::Host;
use hks_t6::value::Value;
use hks_t6::vm::Vm;

/// The row's hardware-profile name (his `omni_movement` setting).
pub const KEY: &str = "bo2_omni";

/// The row's text, in the caps BO2's rows use.
const LABEL: &str = "OMNI MOVEMENT";

/// His setting as the row's profile value.
pub fn profile(s: &frame::GameSettings) -> (String, String) {
    (KEY.to_owned(), if s.omni_movement { "1" } else { "0" }.to_owned())
}

/// After the scripts that load `T6.Menus.OptionsControls` (`T6.Options`
/// requires it; the front end's `T6.Main` and a match's in-game menus load
/// that): its Move tab builds as BO2's, then adds the row. False when the
/// menu is not loaded.
pub fn add(host: &mut Host) -> bool {
    let controls = host.field("CoD", "OptionsControls");
    let Some(table) = controls.table().cloned() else {
        return false;
    };
    let original = table.borrow().get_str("CreateMoveTab");
    if !original.truthy() || table.borrow().get_str("bo2zmOmniRow").truthy() {
        return false;
    }
    let wrapped = host.vm.native("CreateMoveTab", move |vm, a| {
        let made = vm.call(original.clone(), a.clone())?;
        if let Err(e) = add_row(vm, a.first().unwrap_or(&Value::Nil)) {
            diag::warn!(Ui, "bo2zm lui: Omni Movement row: {e}");
        }
        Ok(made)
    });
    let mut t = table.borrow_mut();
    t.set_str("CreateMoveTab", wrapped);
    t.set_str("bo2zmOmniRow", Value::Bool(true));
    true
}

/// The row, on the tab's `buttonList` (CreateMoveTab keeps it on `self`).
fn add_row(vm: &mut Vm, tab: &Value) -> Result<(), hks_t6::vm::LuaError> {
    let list = vm.index(tab, &Value::str("buttonList"))?;
    let cod = vm.global("CoD");
    let button = vm.index(&cod, &Value::str("CoD9Button"))?;
    let height = vm.index(&button, &Value::str("Height"))?.as_num().unwrap_or(0.0);
    let spacer = vm.index(&list, &Value::str("addSpacer"))?;
    vm.call(spacer, vec![list.clone(), Value::Num(height / 2.0)])?;
    let selector = vm.index(&list, &Value::str("addHardwareProfileLeftRightSelector"))?;
    let row = vm.call(selector, vec![list.clone(), Value::str(LABEL), Value::str(KEY)])?;
    let options = vm.index(&cod, &Value::str("Options"))?;
    let on_off = vm.index(&options, &Value::str("Button_AddChoices_OnOrOff"))?;
    vm.call(on_off, vec![row.into_iter().next().unwrap_or(Value::Nil)])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hks_t6::value::Table;

    /// Black Ops II's own menu scripts (the multiplayer front end's rawfile
    /// dump, in the folder BO2_FRONT_SCRIPTS names); none without it.
    fn bo2_scripts() -> Option<Vec<(String, Vec<u8>)>> {
        let mut out = Vec::new();
        for e in std::fs::read_dir(std::env::var_os("BO2_FRONT_SCRIPTS")?).ok()? {
            let p = e.ok()?.path();
            if p.extension().is_some_and(|x| x == "lua") {
                out.push((p.file_name()?.to_string_lossy().into_owned(), std::fs::read(&p).ok()?));
            }
        }
        Some(out)
    }

    /// Build a Move tab with `make` and count its button list's rows.
    fn rows(host: &mut Host, make: Value) -> f32 {
        let tab = Value::Table(Table::new_ref());
        host.call(make, vec![tab.clone(), Value::Num(0.0)]).expect("the Move tab builds");
        let list = host.vm.index(&tab, &Value::str("buttonList")).unwrap();
        let count = host.vm.index(&list, &Value::str("getNumChildren")).unwrap();
        host.call(count, vec![list]).unwrap()[0].as_num().unwrap()
    }

    #[test]
    fn move_tab_ends_with_the_omni_row() {
        let Some(scripts) = bo2_scripts() else {
            return;
        };
        let mut host = Host::new(scripts, Box::new(|_, _, _| 0.0));
        host.load_base();
        host.require("T6.Options");
        let controls = host.field("CoD", "OptionsControls");
        let bo2s = host.vm.index(&controls, &Value::str("CreateMoveTab")).unwrap();
        let before = rows(&mut host, bo2s);
        assert!(add(&mut host));
        assert!(!add(&mut host), "the tab is wrapped once");
        host.values.borrow_mut().profile.insert(KEY.into(), "1".into());
        let ours = host.vm.index(&controls, &Value::str("CreateMoveTab")).unwrap();
        assert_eq!(rows(&mut host, ours), before + 2.0, "a spacer and the row");
        assert!(host.errors.is_empty(), "{:?}", host.errors);
    }
}
