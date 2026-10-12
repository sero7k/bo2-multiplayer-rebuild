//! bo2mp shell: the Zombies screens, drawn with the Multiplayer screens'
//! parts (header, rows, tabs, map tiles, panels) in the Zombies palette
//! (`theme::palette`): the lobby, the map picker and the start picker.

use bevy::prelude::*;

use super::theme::*;
use super::view::{Ctx, Kids, grid_more, grid_window, header, menu_rows, picture_box, short_section, tabs};
use super::zombies::{self, MAPS, ZmState, chosen, game_line, pages, start_name};
use super::{Hit, Row, Screen, Shell};

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    let st = zombies::state();
    match shell.screen {
        Screen::ZmMapPick => map_pick(c, cx, shell, &st),
        Screen::ZmStartPick => start_pick(c, cx, shell, rows),
        _ => lobby(c, cx, shell, rows, &st),
    }
}

fn abs(left: Option<f32>, top: Option<f32>, right: Option<f32>, bottom: Option<f32>, cx: &Ctx) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: left.map_or(Val::Auto, |v| cx.px(v)),
        top: top.map_or(Val::Auto, |v| cx.px(v)),
        right: right.map_or(Val::Auto, |v| cx.px(v)),
        bottom: bottom.map_or(Val::Auto, |v| cx.px(v)),
        ..default()
    }
}

/// The lobby: the game card (its picture, the state, what START plays) and
/// the rows on the left; the map, its starts and the Zombies game's state
/// on the right.
fn lobby(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row], st: &ZmState) {
    header(c, cx, &["ZOMBIES"]);
    let (m, z) = chosen(shell);
    let pictures = zombies::pictures(m, shell.zm_start);
    let round = zombies::ROUNDS[shell.zm_round.min(zombies::ROUNDS.len() - 1)];
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
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
                    let (status, colour) = match (&st.running, st.problem()) {
                        (Some(_), _) => ("IN ZOMBIES", GOLD),
                        (None, Some(_)) => ("Zombies is not installed", cx.pal.accent),
                        (None, None) => ("Ready", WHITE),
                    };
                    t.spawn((Text::new(status), cx.bold(28.0), TextColor(colour)));
                    let line = match &st.running {
                        Some(game) => format!("{game} is running. Quit it to come back here."),
                        None if round > 1 => format!("{}, from round {round}", game_line(m, z)),
                        None => game_line(m, z),
                    };
                    t.spawn((Text::new(line), cx.regular(21.0), TextColor(DIM)));
                });
        });
        menu_rows(col, cx, rows, shell.focus, 600.0, false);
    });
    // The right column: the map and its starts, then the Zombies game.
    c.spawn((
        Node {
            width: cx.px(560.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(cx.px(24.0)),
            row_gap: cx.px(6.0),
            border: UiRect::top(cx.px(3.0)),
            ..abs(None, Some(176.0), Some(64.0), None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        p.spawn((Text::new(m.title), cx.bold(48.0), TextColor(WHITE)));
        p.spawn((Text::new(m.place.to_uppercase()), cx.bold(19.0), TextColor(cx.pal.accent)));
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), margin: UiRect::vertical(cx.px(10.0)), ..default() }, BackgroundColor(cx.pal.line)));
        p.spawn((Text::new("STARTS AND MODES"), cx.bold(19.0), TextColor(FAINT)));
        for (i, s) in m.starts.iter().enumerate() {
            let on = i == shell.zm_start.min(m.starts.len() - 1);
            p.spawn((
                Node { height: cx.px(34.0), column_gap: cx.px(12.0), align_items: AlignItems::Center, padding: UiRect::horizontal(cx.px(10.0)), ..default() },
                BackgroundColor(if on { cx.pal.accent_a(0.16) } else { cx.pal.panel_soft }),
            ))
            .with_children(|r| {
                r.spawn((Node { width: cx.px(4.0), height: cx.px(20.0), ..default() }, BackgroundColor(if on { cx.pal.accent } else { Color::NONE })));
                r.spawn((Node { flex_grow: 1.0, ..default() },)).with_child((Text::new(start_name(m, s)), cx.bold(21.0), TextColor(if on { WHITE } else { TEXT })));
                r.spawn((Text::new(s.mode_title), cx.bold(18.0), TextColor(if on { cx.pal.accent } else { DIM })));
            });
        }
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), margin: UiRect::vertical(cx.px(10.0)), ..default() }, BackgroundColor(cx.pal.line)));
        match st.problem() {
            Some(problem) => {
                p.spawn((Text::new("ZOMBIES IS NOT INSTALLED"), cx.bold(22.0), TextColor(cx.pal.accent)));
                p.spawn((Text::new(problem.to_owned()), cx.regular(20.0), TextColor(TEXT)));
            }
            None => {
                let pack = if st.bo1() { "ZOMBIES DECLASSIFIED" } else { "NOT SET UP" };
                for (k, v) in [("GAME", if st.running.is_some() { "RUNNING" } else { "READY" }), ("BLACK OPS MAPS", pack), ("CHEAT MENU", "NUM 1 IN A SOLO GAME")] {
                    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|r| {
                        r.spawn((Text::new(k), cx.bold(19.0), TextColor(FAINT)));
                        r.spawn((Text::new(v), cx.bold(19.0), TextColor(TEXT)));
                    });
                }
            }
        }
    });
}

/// The map picker: a page per game, the maps as tiles.
fn map_pick(c: &mut Kids, cx: &mut Ctx, shell: &Shell, st: &ZmState) {
    let p = pages(st.bo1());
    let page = shell.page.min(p.len() - 1);
    header(c, cx, &["ZOMBIES", "SELECT MAP", p[page].0]);
    if p.len() > 1 {
        let labels: Vec<String> = p.iter().map(|(t, _)| short_section(t)).collect();
        tabs(c, cx, &labels, page, Hit::Page);
    }
    let maps = &p[page].1;
    let (tw, th, gap) = (280.0, 158.0, 14.0);
    let (cols, first, last) = grid_window(cx, maps.len(), shell.focus, tw, th, gap, 196.0, 190.0);
    grid_more(c, cx, first, last, maps.len(), 72.0 + cols as f32 * (tw + gap) - 220.0, 160.0);
    c.spawn(Node {
        width: cx.px(cols as f32 * (tw + gap)),
        flex_wrap: FlexWrap::Wrap,
        column_gap: cx.px(gap),
        row_gap: cx.px(gap),
        ..abs(Some(72.0), Some(196.0), None, None, cx)
    })
    .with_children(|grid| {
        for (k, &i) in maps.iter().enumerate().take(last).skip(first) {
            let m = &MAPS[i];
            let on = k == shell.focus;
            let chosen = m.map == shell.zm_map;
            grid.spawn((
                Hit::Row(k),
                Button,
                Node { width: cx.px(tw), height: cx.px(th), border: UiRect::all(cx.px(if on { 3.0 } else { 1.0 })), ..default() },
                BorderColor::all(if on { cx.pal.accent } else { cx.pal.line }),
            ))
            .with_children(|tile| {
                let pictures = zombies::pictures(m, 0);
                picture_box(tile, cx, &pictures, tw - if on { 6.0 } else { 2.0 }, th - if on { 6.0 } else { 2.0 }, ());
                tile.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        padding: UiRect::axes(cx.px(10.0), cx.px(5.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    BackgroundGradient::from(LinearGradient::to_bottom(vec![
                        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(0.0)),
                        ColorStop::new(cx.pal.strip, Val::Percent(100.0)),
                    ])),
                ))
                .with_children(|bar| {
                    bar.spawn((Text::new(m.title), cx.bold(21.0), TextColor(if on { WHITE } else { TEXT })));
                    if chosen {
                        bar.spawn((Text::new("●"), cx.bold(16.0), TextColor(cx.pal.accent)));
                    }
                });
            });
        }
    });
    if let Some(&i) = maps.get(shell.focus) {
        let m = &MAPS[i];
        let modes: Vec<&str> = m.starts.iter().map(|s| s.mode_title).fold(Vec::new(), |mut v, t| {
            if !v.contains(&t) {
                v.push(t);
            }
            v
        });
        c.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..abs(None, None, Some(72.0), Some(86.0), cx)
        })
        .with_children(|t| {
            t.spawn((Text::new(m.title), cx.bold(64.0), TextColor(WHITE)));
            t.spawn((Text::new(format!("{}  ·  {}", m.place.to_uppercase(), modes.join(" / "))), cx.bold(22.0), TextColor(cx.pal.accent)));
        });
    }
}

/// The start picker: the map's starts and modes, and the one in focus.
fn start_pick(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    let (m, _) = chosen(shell);
    header(c, cx, &["ZOMBIES", m.title, "SELECT START"]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(6.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
    })
    .with_children(|col| {
        let shown: Vec<Row> = rows.iter().map(|r| Row { sub: String::new(), ..r.clone() }).collect();
        menu_rows(col, cx, &shown, shell.focus, 560.0, false);
    });
    let Some(s) = m.starts.get(shell.focus) else { return };
    let pictures = zombies::pictures(m, shell.focus);
    c.spawn((
        Node {
            width: cx.px(600.0),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(10.0),
            border: UiRect::top(cx.px(3.0)),
            ..abs(None, Some(176.0), Some(64.0), None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        picture_box(p, cx, &pictures, 600.0, 300.0, ());
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(8.0), padding: UiRect::new(cx.px(24.0), cx.px(24.0), cx.px(8.0), cx.px(24.0)), ..default() })
            .with_children(|t| {
                t.spawn((Text::new(start_name(m, s)), cx.bold(48.0), TextColor(WHITE)));
                t.spawn((Text::new(s.mode_title), cx.bold(22.0), TextColor(cx.pal.accent)));
                t.spawn((Text::new(s.desc), cx.regular(23.0), TextColor(TEXT)));
                if shell.focus == shell.zm_start.min(m.starts.len() - 1) {
                    t.spawn((Text::new("SELECTED"), cx.bold(19.0), TextColor(cx.pal.accent)));
                }
            });
    });
}
