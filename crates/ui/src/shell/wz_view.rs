//! bo2mp shell: the WARZONE screens, laid out as Multiplayer's (its menu,
//! the Custom Game lobby, the mode picker) with the same parts (header,
//! rows, tabs, panels) in WARZONE's palette (`wz_theme::PALETTE`, which
//! `theme::palette` hands every WARZONE screen). Create-a-Class, Barracks
//! and Options opened from here are Multiplayer's own screens in the same
//! palette.

use bevy::prelude::*;
use gamemode_iw4::resurgence;

use super::bridge::{self, Catalog};
use super::cards::{self, Kind};
use super::theme::*;
use super::view::{Ctx, Kids, abs, banner, emblem, header, menu_rows, picture_box, tabs};
use super::warzone::{self, MODES};
use super::wz_theme as wt;
use super::{Hit, Row, Screen, Shell, Tab};

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    match shell.screen {
        Screen::WzMenu => menu(c, cx, shell, rows),
        Screen::WzModePick => mode_picker(c, cx, shell, rows),
        _ => lobby(c, cx, shell, cat, rows),
    }
}

/// WARZONE's menu: its rows, and what Resurgence is beside them (where
/// Multiplayer has its online panel).
fn menu(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    header(c, cx, &[wt::HEADER]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
    })
    .with_children(|col| menu_rows(col, cx, rows, shell.focus, 540.0, false));
    rules_panel(c, cx, resurgence::MAP_TITLE, None, 680.0);
}

/// A panel at the right: a title, a line about it, then Resurgence's rules.
fn rules_panel(c: &mut Kids, cx: &mut Ctx, title: &str, about: Option<&str>, width: f32) {
    let r = resurgence::REBIRTH;
    let redeploys: Vec<String> = r
        .phases
        .iter()
        .map(|p| p.redeploy_s.map_or_else(|| "OFF".to_owned(), |s| format!("{s}s")))
        .collect();
    c.spawn((
        Node {
            width: cx.px(width),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(12.0),
            padding: UiRect::all(cx.px(28.0)),
            border: UiRect::top(cx.px(3.0)),
            ..abs(None, Some(176.0), Some(64.0), None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        p.spawn((Text::new(title), cx.bold(48.0), TextColor(WHITE)));
        if let Some(a) = about {
            p.spawn((Text::new(a), cx.regular(23.0), TextColor(TEXT)));
            p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(cx.pal.line)));
        }
        p.spawn((Text::new(resurgence::DISPLAY_NAME), cx.bold(20.0), TextColor(cx.pal.accent)));
        for line in [
            "While a squadmate lives, you redeploy after a wait that grows as the circle closes. A squad with nobody alive is out.".to_owned(),
            format!("{} circles. Redeploy wait by circle: {}.", r.phases.len(), redeploys.join(" / ")),
            format!("Armor: up to {} plates of {}. Hold [Use] to put one in.", r.plate_slots, r.plate_hp),
            "Cash from kills and the ground buys a loadout drop, a self-revive kit, a UAV, plates or a squadmate back at the buy stations.".to_owned(),
        ] {
            p.spawn((Text::new(line), cx.regular(21.0), TextColor(TEXT)));
        }
    });
}

/// The playlists, Resurgence on Rebirth Island first; what the one in focus
/// is at the right.
fn mode_picker(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    let first = if shell.wz_quick { wt::QUICK_PLAY.0 } else { wt::PRIVATE_MATCH.0 };
    header(c, cx, &[first, wt::MODE_HEADER]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(6.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
    })
    .with_children(|col| {
        let shown: Vec<Row> = rows.iter().map(|r| Row { sub: String::new(), ..r.clone() }).collect();
        // (The first is the main playlist; the presets follow.)
        col.spawn((Text::new(wt::FEATURED), cx.bold(18.0), TextColor(cx.pal.accent)));
        menu_rows(col, cx, &shown, shell.focus, 620.0, false);
    });
    if let Some(m) = MODES.get(shell.focus) {
        let about = if shell.focus == shell.wz_mode && !shell.wz_quick { format!("{}\n\nSELECTED", m.desc) } else { m.desc.to_owned() };
        rules_panel(c, cx, m.title, Some(&about), 640.0);
    }
}

/// The private match's squad lobby: the match card and the tab's rows at
/// the left, the squads at the right.
fn lobby(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    header(c, cx, &wt::LOBBY_HEADER);
    let labels: Vec<String> = wt::TABS.iter().map(|t| (*t).to_owned()).collect();
    let on = Tab::ALL.iter().position(|t| *t == shell.tab).unwrap_or(0);
    tabs(c, cx, &labels, on, Hit::Tab);
    let (map_title, pictures) = match warzone::map(cat) {
        Ok((ui, title)) => (title, cat.map(&ui).map(|m| m.pictures.clone()).unwrap_or_default()),
        Err(_) => (resurgence::MAP_TITLE.to_owned(), Vec::new()),
    };
    let starting = bridge::starting_in();
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(196.0), None, None, cx)
    })
    .with_children(|col| {
        col.spawn((
            Node { width: cx.px(600.0), flex_direction: FlexDirection::Column, border: UiRect::left(cx.px(4.0)), margin: UiRect::bottom(cx.px(14.0)), ..default() },
            BorderColor::all(cx.pal.accent),
            BackgroundColor(cx.pal.panel),
        ))
        .with_children(|card| {
            picture_box(card, cx, &pictures, 596.0, 236.0, ());
            card.spawn(Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(cx.px(16.0), cx.px(12.0)), row_gap: cx.px(2.0), ..default() })
                .with_children(|t| {
                    let status = match starting {
                        Some(ms) => format!("{} {}", wt::DEPLOYING_IN, (ms / 1000.0).ceil() as i32),
                        None => wt::READY.to_owned(),
                    };
                    t.spawn((Text::new(status), cx.bold(28.0), TextColor(if starting.is_some() { cx.pal.accent } else { WHITE })));
                    t.spawn((Text::new(format!("{} on {}", warzone::mode_line(shell), map_title.to_uppercase())), cx.regular(21.0), TextColor(DIM)));
                });
        });
        menu_rows(col, cx, rows, shell.focus, 600.0, false);
    });
    squad_list(c, cx, shell, cat);
}

/// Who is in, squad by squad: his squad in full, the others a line each.
fn squad_list(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog) {
    let size = warzone::squad_size(shell);
    let squads = warzone::squads(shell);
    let total = u32::from(size) * squads;
    c.spawn(Node {
        width: cx.px(560.0),
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(4.0),
        ..abs(None, Some(176.0), Some(64.0), None, cx)
    })
    .with_children(|list| {
        list.spawn((
            Text::new(format!("{squads} Squads  ·  {total} Players ({} Max)", warzone::MAX_PLAYERS)),
            cx.regular(19.0),
            TextColor(DIM),
            Node { margin: UiRect::bottom(cx.px(6.0)), ..default() },
        ));
        let squad_head = |list: &mut Kids, cx: &Ctx, label: String, color: Color| {
            list.spawn((Text::new(label), cx.bold(18.0), TextColor(color), Node { margin: UiRect::top(cx.px(6.0)), ..default() }));
        };
        squad_head(list, cx, wt::MY_SQUAD.to_owned(), cx.pal.accent);
        let me = &cat.player;
        let mut members = vec![(me.level.to_string(), me.name.clone(), true, cards::mine(Kind::Card, &me.card), cards::mine(Kind::Emblem, &me.emblem))];
        for n in 1..u32::from(size) {
            let key = format!("Bot {n}");
            let name = sim::lobby_bots::lobby_bot_name(n);
            let level = sim::lobby_bots::level_for(&name);
            members.push((level.to_string(), name, false, cards::for_name(Kind::Card, &key), cards::for_name(Kind::Emblem, &key)));
        }
        for (level, name, mine, card, mark) in members {
            list.spawn((
                Node { height: cx.px(36.0), column_gap: cx.px(10.0), align_items: AlignItems::Center, padding: UiRect::right(cx.px(10.0)), ..default() },
                BackgroundColor(if mine { cx.pal.accent_a(0.14) } else { cx.pal.panel_soft }),
            ))
            .with_children(|r| {
                r.spawn((Node { width: cx.px(4.0), height: cx.px(36.0), ..default() }, BackgroundColor(cx.pal.accent)));
                emblem(r, cx, &mark, 32.0);
                r.spawn((Node { width: cx.px(34.0), ..default() },)).with_child((Text::new(level), cx.bold(20.0), TextColor(WHITE)));
                r.spawn((Node { flex_grow: 1.0, ..default() },)).with_child((Text::new(name), cx.bold(21.0), TextColor(if mine { GOLD } else { TEXT })));
                banner(r, cx, &card, 128.0, 30.0, None);
            });
        }
        if squads > 1 {
            squad_head(list, cx, wt::ENEMY_SQUADS.to_owned(), OTHER);
        }
        // (The bots after his squad's, numbered on.)
        let mut next = u32::from(size);
        for k in 2..=squads {
            let names: Vec<String> = (0..u32::from(size))
                .map(|i| sim::lobby_bots::lobby_bot_name(next + i))
                .collect();
            next += u32::from(size);
            list.spawn((
                Node { height: cx.px(30.0), column_gap: cx.px(12.0), align_items: AlignItems::Center, padding: UiRect::right(cx.px(10.0)), ..default() },
                BackgroundColor(cx.pal.panel_soft),
            ))
            .with_children(|r| {
                r.spawn((Node { width: cx.px(4.0), height: cx.px(30.0), ..default() }, BackgroundColor(OTHER)));
                r.spawn((Node { width: cx.px(96.0), ..default() },)).with_child((Text::new(format!("{} {k}", wt::SQUAD)), cx.bold(18.0), TextColor(DIM)));
                r.spawn((Node { flex_grow: 1.0, overflow: Overflow::clip(), ..default() },))
                    .with_child((Text::new(names.join("  ·  ")), cx.regular(18.0), TextColor(TEXT)));
            });
        }
    });
}

const OTHER: Color = wt::OTHER_SQUAD;
