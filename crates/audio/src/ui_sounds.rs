//! Black Ops' interface sounds for Black Ops II's menus and match UI: a BO2
//! UI sound is played as Black Ops' counterpart (`uin_navigation_*`) when the
//! bank carries it, and as itself otherwise. The pre-match countdown, which
//! Black Ops had no sound for, plays BO2's own tick when the match bank has
//! it and Modern Warfare 3's otherwise.

use asset_audio::SoundCatalog;
use asset_core::AssetNamespace;

use AssetNamespace::{Iw4, Iw5, T5, T6};

/// BO2 UI sound → what plays in its place, first one the bank has.
pub(crate) const UI_ROUTES: &[(&str, &[(AssetNamespace, &str)])] = &[
    // Moving the highlight.
    // (bo2mp: Modern Warfare 3's menu sounds first, then MW2's, then
    // Black Ops'.)
    ("uin_navigation_click", &[(Iw5, "mouse_over"), (Iw4, "mouse_over"), (T5, "uin_navigation_over"), (T5, "mouse_over")]),
    ("cac_grid_nav", &[(Iw5, "mouse_over"), (Iw4, "mouse_over"), (T5, "uin_navigation_over"), (T5, "mouse_over")]),
    ("uin_main_nav", &[(Iw5, "mouse_over"), (Iw4, "mouse_over"), (T5, "uin_navigation_over"), (T5, "mouse_over")]),
    // Choosing.
    ("uin_navigation_select_main", &[(Iw5, "mouse_click"), (Iw4, "mouse_click"), (T5, "uin_navigation_click"), (T5, "mouse_click")]),
    ("uin_main_enter", &[(T5, "uin_navigation_menu_lg_open"), (T5, "uin_navigation_click")]),
    ("cac_enter_cac", &[(T5, "uin_navigation_menu_lg_open"), (T5, "uin_navigation_click")]),
    ("cac_loadout_edit_sel", &[(T5, "uin_navigation_click"), (T5, "mouse_click")]),
    ("cac_loadout_edit_submenu", &[(T5, "uin_navigation_menu_sm_open"), (T5, "uin_navigation_click")]),
    ("cac_equipment_add", &[(T5, "uin_navigation_click"), (T5, "mouse_click")]),
    ("cac_grid_equip_item", &[(T5, "uin_navigation_click"), (T5, "mouse_click")]),
    ("cac_slide_equip_item", &[(T5, "uin_navigation_click"), (T5, "mouse_click")]),
    // Backing out, and what cannot be taken.
    ("cac_cmn_backout", &[(Iw5, "mp_ui_decline"), (Iw4, "mp_ui_decline"), (Iw5, "mouse_click"), (T5, "uin_navigation_backout"), (T5, "uin_navigation_menu_sm_close")]),
    ("uin_main_exit", &[(T5, "uin_navigation_backout"), (T5, "uin_navigation_menu_lg_close")]),
    ("cac_restricted", &[(T5, "uin_navigation_backout")]),
    ("cac_equipment_remove", &[(T5, "uin_navigation_backout")]),
    // Pause and popups.
    ("uin_main_pause", &[(T5, "uin_navigation_sys_open")]),
    ("uin_alert_slideout", &[(T5, "uin_navigation_submenu_over")]),
    // The countdown before a match.
    ("uin_start_count_down", &[(T6, "uin_start_count_down"), (Iw5, "match_countdown_tick")]),
    // bo2mp: Black Ops' notification sounds, as its `maps/mp/_popups.gsc`
    // plays them with each popup (`displayMedal(.., "uin_challenge_repeatable")`,
    // `displayRankUp(.., "mus_level_up")`, `displayChallengeComplete(..,
    // "mus_challenge_complete")`, contracts `mus_contract_complete` /
    // `mus_contract_expire`), all in BO1's `common_mp`. The HUD asks for
    // them by these names (`hud_medals`); BO2's own sound stands in when
    // Black Ops is not there.
    (BO1_MEDAL, &[(T5, "uin_challenge_repeatable"), (T6, "mus_lau_medal")]),
    (BO1_RANK_UP, &[(T5, "mus_level_up")]),
    (BO1_CHALLENGE, &[(T5, "mus_challenge_complete")]),
    // BO2's LUI medal popup (the HUD without the modern medal box) plays
    // Black Ops' medal sound too.
    ("mus_lau_medal", &[(T5, "uin_challenge_repeatable"), (T6, "mus_lau_medal")]),
    // (Guessed: BO2's LUI rank-up sting; not yet seen played in a log.)
    ("mus_lau_rank_up", &[(T5, "mus_level_up"), (T6, "mus_lau_rank_up")]),
];

/// The HUD's names for Black Ops' notification sounds (routed above;
/// `ui::hud_medals` writes `BO1_MEDAL` as a literal: the UI crate does not
/// depend on this one).
pub(crate) const BO1_MEDAL: &str = "bo2mp_bo1_medal";
pub(crate) const BO1_RANK_UP: &str = "bo2mp_bo1_rank_up";
pub(crate) const BO1_CHALLENGE: &str = "bo2mp_bo1_challenge";

/// Black Ops' aliases in the bank that look like notification sounds (for
/// the log: the names are read from its zones, not known beforehand).
pub(crate) fn bo1_notification_candidates(bank: &SoundCatalog) -> Vec<String> {
    const WORDS: &[&str] = &["medal", "card", "challenge", "rank", "notif", "splash", "level_up", "contract", "popup", "promot"];
    let mut out: Vec<String> = (0..bank.sounds.len())
        .filter(|&i| bank.namespace_of_alias(i) == T5)
        .filter_map(|i| bank.name_at(i))
        .map(str::to_ascii_lowercase)
        .filter(|n| WORDS.iter().any(|w| n.contains(w)))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The sound a BO2 UI sound plays as, when the bank has a stand-in.
pub(crate) fn ui_sound(bank: &SoundCatalog, alias: &str) -> Option<(AssetNamespace, &'static str)> {
    let (_, to) = UI_ROUTES.iter().find(|(from, _)| from.eq_ignore_ascii_case(alias))?;
    to.iter().copied().find(|(ns, name)| bank.index_in(*ns, name).is_some())
}
