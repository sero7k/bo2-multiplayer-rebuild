//! bo2mp shell: FIND A MATCH. Public matches by playlist, as Black Ops II's
//! own filter does it: two tabs, CORE (the standard modes) and PARTY GAMES
//! (Gun Game, One in the Chamber, Sharpshooter, Sticks and Stones); each
//! mode a row to tick, its playlist picture, description and rules (from
//! the game's own `mp/gamesettings_<type>.cfg`) beside it; then SEARCH,
//! which lists the server's open lobbies of the modes ticked (FIND A
//! LOBBY's list, filtered). None found: host one of those modes, so the
//! next player searching finds yours.
//!
//! BRAINROT 24/7 (top of CORE): Shipment, Call of Duty 4's smallest map, and
//! nothing else, in whichever modes are ticked (Team Deathmatch when none).

use bevy::prelude::*;

use super::bridge::{Catalog, ModeEntry};
use super::theme::*;
use super::view::{Ctx, Kids, abs, header, menu_rows, tabs};
use super::{Act, Hit, Row, Shell};

/// The filter's tabs.
pub(crate) const TABS: [&str; 2] = ["CORE", "PARTY GAMES"];

/// Black Ops II's party games (its PARTY GAMES playlist).
const PARTY: [&str; 4] = ["gun", "oic", "shrp", "sas"];

pub(crate) fn is_party(key: &str) -> bool {
    PARTY.contains(&key)
}

/// The modes on a tab, in the catalog's order: (index in the catalog, mode).
pub(crate) fn modes(c: &Catalog, tab: usize) -> Vec<(usize, &ModeEntry)> {
    c.modes.iter().enumerate().filter(|(_, m)| is_party(&m.key) == (tab == 1)).collect()
}

/// Black Ops II's playlist picture for a mode (`playlist_tdm`, ...).
pub(crate) fn picture(key: &str) -> Vec<String> {
    let own = match key {
        "tdm" => "playlist_tdm",
        "dm" => "playlist_ffa",
        "dom" => "playlist_domination",
        "sd" => "playlist_search_destroy",
        "conf" => "playlist_kill_confirm",
        "hq" => "playlist_headquarters",
        "koth" => "playlist_koth",
        "ctf" => "playlist_ctf",
        "dem" => "playlist_demolition",
        "gun" => "playlist_gungame",
        "oic" => "playlist_one_chamber",
        "shrp" => "playlist_sharpshooter",
        "sas" => "playlist_sticks_and_stones",
        _ => "",
    };
    [own.to_owned(), format!("playlist_{key}"), if is_party(key) { "playlist_party" } else { "playlist_core" }.to_owned()]
        .into_iter()
        .filter(|n| !n.is_empty())
        .collect()
}

/// Shipment, as the catalog lists it (Call of Duty 4's `mp_shipment`).
pub(crate) fn shipment(c: &Catalog) -> Option<&super::bridge::MapEntry> {
    c.maps.iter().find(|m| is_shipment(&m.ui_name))
}

/// A map name (`iw3:mp_shipment`, `mp_shipment_iw3`) that is Shipment.
pub(crate) fn is_shipment(map: &str) -> bool {
    let stem = map.split(':').next_back().unwrap_or(map).to_ascii_lowercase();
    stem == "mp_shipment" || stem.starts_with("mp_shipment_") && !stem.starts_with("mp_shipment_long")
}

/// The modes ticked, as their titles.
fn ticked_titles(shell: &Shell, c: &Catalog) -> Vec<String> {
    shell.filter.iter().filter_map(|k| c.mode(k).map(|m| m.title.clone())).collect()
}

pub(crate) fn rows(shell: &Shell, c: &Catalog) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::new();
    if shell.page == 0 {
        out.push(Row {
            label: "BRAINROT 24/7".to_owned(),
            sub: "Shipment. All day, every day.".to_owned(),
            value: Some(if shell.brainrot { "ON".to_owned() } else { "OFF".to_owned() }),
            act: Act::Brainrot,
        });
    }
    out.extend(modes(c, shell.page)
        .into_iter()
        .map(|(i, m)| Row {
            label: m.title.clone(),
            sub: m.desc.clone(),
            value: Some(if shell.filter.contains(&m.key) { "ON".to_owned() } else { "OFF".to_owned() }),
            act: Act::FilterMode(i),
        }));
    let picked = ticked_titles(shell, c);
    let picked: Vec<String> = if shell.brainrot {
        let modes = if picked.is_empty() { vec!["TEAM DEATHMATCH".to_owned()] } else { picked };
        modes.into_iter().map(|m| format!("{m} ON SHIPMENT")).collect()
    } else {
        picked
    };
    out.push(Row {
        label: "SEARCH".to_owned(),
        sub: if picked.is_empty() {
            "Every mode: tick the ones you want to play, or search them all.".to_owned()
        } else {
            format!("Open lobbies playing {}.", picked.join(", "))
        },
        value: None,
        act: Act::SearchMatches,
    });
    out
}

/// Enter on a mode: tick it, or untick it.
pub(crate) fn toggle(shell: &mut Shell, c: &Catalog, i: usize) {
    let Some(m) = c.modes.get(i) else { return };
    if let Some(at) = shell.filter.iter().position(|k| *k == m.key) {
        shell.filter.remove(at);
    } else {
        shell.filter.push(m.key.clone());
    }
}

/// Whether a lobby passes the filter: its mode is one ticked (nothing
/// ticked: every mode; BRAINROT 24/7 with none ticked: Team Deathmatch),
/// and with BRAINROT 24/7 on, its map is Shipment.
pub(crate) fn passes(shell: &Shell, map: &str, mode: &str) -> bool {
    let mode_ok = if shell.filter.is_empty() {
        !shell.brainrot || mode.eq_ignore_ascii_case("tdm")
    } else {
        shell.filter.iter().any(|k| k.eq_ignore_ascii_case(mode))
    };
    mode_ok && (!shell.brainrot || is_shipment(map))
}

/// The mode to host when the search finds nothing: the first ticked, or
/// the lobby's own.
pub(crate) fn host_mode(shell: &Shell) -> String {
    shell.filter.first().cloned().unwrap_or_else(|| if shell.brainrot { "tdm".to_owned() } else { shell.mode.clone() })
}

fn minutes(v: f32) -> String {
    if v <= 0.0 {
        "UNLIMITED".to_owned()
    } else if v.fract() == 0.0 {
        format!("{v:.0} MIN")
    } else {
        format!("{v:.1} MIN")
    }
}

fn count(v: f32) -> String {
    if v <= 0.0 { "UNLIMITED".to_owned() } else { format!("{v:.0}") }
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    header(c, cx, &["FIND A MATCH", "PLAYLIST FILTER"]);
    let labels: Vec<String> = TABS.iter().map(|t| (*t).to_owned()).collect();
    tabs(c, cx, &labels, shell.page.min(TABS.len() - 1), Hit::Page);
    // (How many are ticked, under the tabs: Black Ops II's "Selected filter".)
    let picked = shell.filter.len();
    c.spawn((
        Node { column_gap: cx.px(10.0), align_items: AlignItems::Center, padding: UiRect::axes(cx.px(14.0), cx.px(5.0)), border: UiRect::left(cx.px(3.0)), ..abs(Some(72.0), Some(176.0), None, None, cx) },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(GOLD),
    ))
    .with_child((
        Text::new(match (picked, shell.brainrot) {
            (0, false) => "ALL MODES".to_owned(),
            (0, true) => "BRAINROT 24/7".to_owned(),
            (n, false) => format!("{n} SELECTED"),
            (n, true) => format!("BRAINROT 24/7  ·  {n} SELECTED"),
        }),
        cx.bold(18.0),
        TextColor(WHITE),
    ));
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(6.0), ..abs(Some(72.0), Some(222.0), None, None, cx) })
        .with_children(|col| {
            let shown: Vec<Row> = rows.iter().map(|r| Row { sub: String::new(), ..r.clone() }).collect();
            menu_rows(col, cx, &shown, shell.focus, 560.0, false);
        });
    // The mode in focus: its picture, what it is, its rules.
    let focused = rows.get(shell.focus).map(|r| r.act);
    c.spawn((
        Node {
            width: cx.px(760.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(cx.px(26.0)),
            row_gap: cx.px(14.0),
            border: UiRect::top(cx.px(3.0)),
            ..abs(Some(680.0), Some(222.0), None, None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| match focused {
        Some(Act::Brainrot) => {
            let map = shipment(cat);
            if let Some(m) = map
                && let Some(pic) = cx.pic(&m.pictures)
            {
                let size = cx.pic_size(&pic);
                let w = 708.0;
                p.spawn((ImageNode::new(pic), Node { width: cx.px(w), height: cx.px(w * size.y / size.x.max(1.0)), ..default() }));
            }
            p.spawn((Text::new("BRAINROT 24/7"), cx.bold(40.0), TextColor(WHITE)));
            p.spawn((
                Text::new("Shipment. All day, every day. Team Deathmatch, or the modes you tick."),
                cx.regular(21.0),
                TextColor(TEXT),
            ));
            p.spawn(Node { column_gap: cx.px(46.0), ..default() }).with_children(|row| {
                for (label, value) in [
                    ("MAP", map.map_or_else(|| "SHIPMENT (CALL OF DUTY 4 NOT FOUND)".to_owned(), |m| m.title.to_uppercase())),
                    ("MODES", if shell.filter.is_empty() { "TEAM DEATHMATCH".to_owned() } else { format!("{} TICKED", shell.filter.len()) }),
                ] {
                    row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(2.0), ..default() }).with_children(|f| {
                        f.spawn((Text::new(label), cx.bold(18.0), TextColor(FAINT)));
                        f.spawn((Text::new(value), cx.bold(26.0), TextColor(WHITE)));
                    });
                }
            });
            let on = shell.brainrot;
            p.spawn((
                Node { padding: UiRect::axes(cx.px(12.0), cx.px(4.0)), border: UiRect::left(cx.px(3.0)), align_self: AlignSelf::FlexStart, ..default() },
                BackgroundColor(if on { cx.pal.accent_soft.with_alpha(0.18) } else { Color::NONE }),
                BorderColor::all(if on { GOLD } else { cx.pal.line }),
            ))
            .with_child((Text::new(if on { "SELECTED" } else { "PRESS ENTER TO SELECT" }), cx.bold(18.0), TextColor(if on { WHITE } else { DIM })));
        }
        Some(Act::FilterMode(i)) if cat.modes.get(i).is_some() => {
            let m = &cat.modes[i];
            p.spawn(Node { column_gap: cx.px(20.0), align_items: AlignItems::Center, ..default() }).with_children(|top| {
                if let Some(pic) = cx.pic(&picture(&m.key)) {
                    top.spawn((ImageNode::new(pic), Node { width: cx.px(96.0), height: cx.px(96.0), flex_shrink: 0.0, ..default() }));
                }
                top.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(4.0), ..default() }).with_children(|t| {
                    t.spawn((Text::new(m.title.clone()), cx.bold(40.0), TextColor(WHITE)));
                    t.spawn((Text::new(m.desc.clone()), cx.regular(21.0), TextColor(TEXT)));
                });
            });
            p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(cx.pal.line)));
            let r = m.rules;
            let mut facts: Vec<(&str, String)> = Vec::new();
            if let Some(t) = r.time_limit {
                facts.push(("MATCH TIME", minutes(t)));
            }
            if let Some(s) = r.score_limit {
                facts.push(("SCORE LIMIT", count(s)));
            }
            if let Some(n) = r.round_limit.filter(|n| *n > 1.0) {
                facts.push(("ROUNDS", count(n)));
            }
            if let Some(n) = r.lives.filter(|n| *n > 0.0) {
                facts.push(("LIVES", count(n)));
            }
            p.spawn(Node { column_gap: cx.px(46.0), ..default() }).with_children(|row| {
                for (label, value) in facts {
                    row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(2.0), ..default() }).with_children(|f| {
                        f.spawn((Text::new(label), cx.bold(18.0), TextColor(FAINT)));
                        f.spawn((Text::new(value), cx.bold(26.0), TextColor(WHITE)));
                    });
                }
            });
            let on = shell.filter.contains(&m.key);
            p.spawn((
                Node { padding: UiRect::axes(cx.px(12.0), cx.px(4.0)), border: UiRect::left(cx.px(3.0)), align_self: AlignSelf::FlexStart, ..default() },
                BackgroundColor(if on { cx.pal.accent_soft.with_alpha(0.18) } else { Color::NONE }),
                BorderColor::all(if on { GOLD } else { cx.pal.line }),
            ))
            .with_child((Text::new(if on { "SELECTED" } else { "PRESS ENTER TO SELECT" }), cx.bold(18.0), TextColor(if on { WHITE } else { DIM })));
        }
        _ => {
            p.spawn((Text::new("SEARCH"), cx.bold(40.0), TextColor(WHITE)));
            let picked = ticked_titles(shell, cat);
            p.spawn((
                Text::new(if picked.is_empty() { "Every mode, Core and Party Games.".to_owned() } else { picked.join("  ·  ") }),
                cx.regular(22.0),
                TextColor(TEXT),
            ));
            p.spawn((
                Text::new("Finds open lobbies on the server playing these modes. None open: you host one, and the next player who searches finds yours."),
                cx.regular(20.0),
                TextColor(DIM),
            ));
        }
    });
}
