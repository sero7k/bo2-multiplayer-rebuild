//! Modern Warfare 3's interface sounds for Black Ops II's menus and match UI:
//! a BO2 UI sound is played as its MW3 counterpart when the bank carries it
//! (MW3's `localized_code_post_gfx_mp`, read from the voice-only folder), and
//! as itself otherwise.

use asset_audio::SoundCatalog;
use asset_core::AssetNamespace;

/// BO2 UI sound → MW3 UI sound.
pub(crate) const MW3_UI: &[(&str, &str)] = &[
    ("uin_navigation_click", "mouse_over"),
    ("cac_grid_nav", "mouse_over"),
    ("uin_navigation_select_main", "mouse_click"),
    ("cac_enter_cac", "mouse_click"),
    ("cac_loadout_edit_sel", "mouse_click"),
    ("cac_loadout_edit_submenu", "mouse_click"),
    ("cac_cmn_backout", "mouse_click"),
    ("cac_equipment_add", "mouse_click"),
    ("cac_grid_equip_item", "mouse_click"),
    ("cac_slide_equip_item", "mouse_click"),
    ("cac_restricted", "mouse_negative"),
    ("cac_equipment_remove", "mouse_negative"),
    ("uin_alert_slideout", "mouse_submenu_over"),
    ("uin_start_count_down", "match_countdown_tick"),
];

/// The MW3 sound a BO2 UI sound plays as, when the bank has it.
pub(crate) fn mw3_ui_sound(bank: &SoundCatalog, alias: &str) -> Option<&'static str> {
    let &(_, to) = MW3_UI
        .iter()
        .find(|(from, _)| from.eq_ignore_ascii_case(alias))?;
    bank.index_in(AssetNamespace::Iw5, to)?;
    Some(to)
}
