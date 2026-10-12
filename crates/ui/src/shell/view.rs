//! bo2mp shell: the screens, drawn again whenever what they show changes
//! (the screen, the row in focus, the lobby's choices, a picture coming in,
//! the online numbers, the window's size).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::bridge::{Catalog, PlayerCard};
use super::online::{OnlineInfo, Status};
use super::pics::{ShellPics, cover};
use super::theme::*;
use super::{COLUMNS, DIFFICULTY, Hit, Row, Screen, Shell, Tab, bridge, pages, rows};

#[derive(Component)]
pub(crate) struct ViewRoot;

type Kids<'a> = ChildSpawnerCommands<'a>;

/// What the drawing needs besides the shell's state.
struct Ctx<'a> {
    s: f32,
    fonts: &'a ShellFonts,
    pics: &'a mut ShellPics,
    icons: Option<&'a assets::T6HudIcons>,
    images: &'a mut Assets<Image>,
    pad: bool,
}

impl Ctx<'_> {
    fn px(&self, v: f32) -> Val {
        Val::Px(v * self.s)
    }

    fn bold(&self, px: f32) -> TextFont {
        font(&self.fonts.bold, px * self.s)
    }

    fn regular(&self, px: f32) -> TextFont {
        font(&self.fonts.regular, px * self.s)
    }

    fn pic(&mut self, names: &[String]) -> Option<Handle<Image>> {
        self.pics.first(names, self.icons, self.images)
    }

    fn pic_size(&self, h: &Handle<Image>) -> Vec2 {
        self.images.get(h).map_or(Vec2::new(16.0, 9.0), |i| i.size_f32())
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

/// The title at the top left: a blue bar, then the trail of screens, the
/// last one bright.
fn header(c: &mut Kids, cx: &Ctx, trail: &[&str]) {
    c.spawn((
        Node {
            column_gap: cx.px(14.0),
            align_items: AlignItems::Center,
            ..abs(Some(72.0), Some(54.0), None, None, cx)
        },
    ))
    .with_children(|h| {
        h.spawn((Node { width: cx.px(5.0), height: cx.px(40.0), ..default() }, BackgroundColor(ACCENT)));
        for (i, part) in trail.iter().enumerate() {
            let last = i + 1 == trail.len();
            if i > 0 {
                h.spawn((Text::new("›"), cx.regular(40.0), TextColor(FAINT)));
            }
            h.spawn((
                Text::new(*part),
                if last { cx.bold(46.0) } else { cx.regular(46.0) },
                TextColor(if last { WHITE } else { DIM }),
            ));
        }
    });
}

/// His card at the top right: rank icon, name, level and how far into it.
fn player_card(c: &mut Kids, cx: &mut Ctx, card: &PlayerCard) {
    let icon = cx.pic(&[card.rank_picture.clone()]);
    c.spawn((
        Node {
            width: cx.px(430.0),
            height: cx.px(88.0),
            padding: UiRect::axes(cx.px(16.0), cx.px(12.0)),
            column_gap: cx.px(16.0),
            align_items: AlignItems::Center,
            border: UiRect::left(cx.px(3.0)),
            ..abs(None, Some(42.0), Some(64.0), None, cx)
        },
        BackgroundColor(PANEL),
        BorderColor::all(ACCENT),
    ))
    .with_children(|p| {
        match icon {
            Some(h) => {
                p.spawn((ImageNode::new(h), Node { width: cx.px(60.0), height: cx.px(60.0), ..default() }));
            }
            None => {
                p.spawn((Node { width: cx.px(60.0), height: cx.px(60.0), ..default() }, BackgroundColor(KEYCAP)));
            }
        }
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            row_gap: cx.px(4.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((Text::new(card.name.clone()), cx.bold(30.0), TextColor(WHITE)));
            let prestige = if card.prestige > 0 { format!("PRESTIGE {}  ·  ", card.prestige) } else { String::new() };
            col.spawn((Text::new(format!("{prestige}LEVEL {}", card.level)), cx.regular(19.0), TextColor(DIM)));
            col.spawn((Node { width: Val::Percent(100.0), height: cx.px(4.0), ..default() }, BackgroundColor(KEYCAP)))
                .with_child((
                    Node { width: Val::Percent(card.xp_frac * 100.0), height: Val::Percent(100.0), ..default() },
                    BackgroundColor(ACCENT),
                ));
        });
        p.spawn((Text::new(card.level.to_string()), cx.bold(44.0), TextColor(WHITE)));
    });
}

/// A key hint: the key (or the pad's button picture) and what it does.
fn hint(c: &mut Kids, cx: &mut Ctx, key: &str, pad: &str, what: &str) {
    let icon = if cx.pad && !pad.is_empty() { cx.pic(&[pad.to_owned()]) } else { None };
    c.spawn(Node { column_gap: cx.px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|h| {
        match icon {
            Some(img) => {
                h.spawn((ImageNode::new(img), Node { width: cx.px(26.0), height: cx.px(26.0), ..default() }));
            }
            None if !key.is_empty() => {
                h.spawn((
                    Node { padding: UiRect::axes(cx.px(7.0), cx.px(1.0)), ..default() },
                    BackgroundColor(KEYCAP),
                ))
                .with_child((Text::new(key), cx.bold(17.0), TextColor(WHITE)));
            }
            None => {}
        }
        h.spawn((Text::new(what), cx.bold(18.0), TextColor(DIM)));
    });
}

fn footer(c: &mut Kids, cx: &mut Ctx, shell: &Shell, online: &OnlineInfo) {
    c.spawn((
        Node {
            width: Val::Percent(100.0),
            height: cx.px(58.0),
            padding: UiRect::horizontal(cx.px(72.0)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border: UiRect::top(Val::Px(1.0)),
            ..abs(Some(0.0), None, None, Some(0.0), cx)
        },
        BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.72)),
        BorderColor::all(LINE),
    ))
    .with_children(|f| {
        f.spawn((Node { column_gap: cx.px(28.0), align_items: AlignItems::Center, ..default() }, Hit::Back, Button))
            .with_children(|l| {
                if shell.screen != Screen::Main {
                    let back = if shell.screen == Screen::Lobby { "LEAVE LOBBY" } else { "BACK" };
                    hint(l, cx, "ESC", "xenonbutton_b", back);
                }
                hint(l, cx, "ENTER", "xenonbutton_a", "SELECT");
                match shell.screen {
                    Screen::Lobby => hint(l, cx, "Q / E", "xenonbutton_lb", "TABS"),
                    Screen::MapPick => hint(l, cx, "Q / E", "xenonbutton_lb", "SECTIONS"),
                    _ => {}
                }
                if shell.screen == Screen::Lobby && shell.tab == Tab::Settings {
                    hint(l, cx, "A / D", "xenonbutton_dpad_rl", "CHANGE");
                }
            });
        f.spawn(Node { column_gap: cx.px(28.0), align_items: AlignItems::Center, ..default() }).with_children(|r| {
            if !cx.pad {
                hint(r, cx, "F10", "", "GAME FOLDERS");
            }
            let (dot, label) = match online.status {
                Status::Online => (GREEN, format!("ONLINE  ·  {} PLAYERS", fmt_count(online.players))),
                Status::Connecting => (GOLD, "CONNECTING".to_owned()),
                Status::Unreachable => (ENEMY, "SERVER UNREACHABLE".to_owned()),
                Status::Offline => (FAINT, "OFFLINE  ·  LOCAL PLAY".to_owned()),
            };
            r.spawn(Node { column_gap: cx.px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|s| {
                s.spawn((
                    Node { width: cx.px(9.0), height: cx.px(9.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() },
                    BackgroundColor(dot),
                ));
                s.spawn((Text::new(label), cx.bold(18.0), TextColor(DIM)));
            });
        });
    });
}

fn fmt_count(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// A column of menu rows. `big`: the main menu's large rows.
fn menu_rows(c: &mut Kids, cx: &Ctx, rows: &[Row], focus: usize, width: f32, big: bool) {
    let (h, px) = if big { (64.0, 40.0) } else { (56.0, 28.0) };
    for (i, r) in rows.iter().enumerate() {
        let on = i == focus;
        let mut e = c.spawn((
            Hit::Row(i),
            Button,
            Node {
                width: cx.px(width),
                min_height: cx.px(h),
                padding: UiRect::new(cx.px(20.0), cx.px(18.0), cx.px(6.0), cx.px(6.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                border: UiRect::left(cx.px(4.0)),
                ..default()
            },
            BorderColor::all(if on { ACCENT } else { Color::NONE }),
        ));
        if on {
            e.insert(fade_right(Color::srgba(0.26, 0.62, 0.95, 0.30), Color::srgba(0.26, 0.62, 0.95, 0.0)));
        } else if !big {
            e.insert(BackgroundColor(PANEL_SOFT));
        }
        e.with_children(|row| {
            row.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|line| {
                line.spawn((Text::new(r.label.clone()), cx.bold(px), TextColor(if on { WHITE } else { TEXT.with_alpha(0.72) })));
                match &r.value {
                    Some(v) => {
                        line.spawn(Node { column_gap: cx.px(14.0), align_items: AlignItems::Center, ..default() }).with_children(|sel| {
                            sel.spawn((Hit::Step(i, -1), Button, Text::new("‹"), cx.bold(px), TextColor(if on { ACCENT } else { FAINT })));
                            sel.spawn((
                                Node { min_width: cx.px(170.0), justify_content: JustifyContent::Center, ..default() },
                            ))
                            .with_child((Text::new(v.clone()), cx.bold(px * 0.85), TextColor(if on { WHITE } else { DIM })));
                            sel.spawn((Hit::Step(i, 1), Button, Text::new("›"), cx.bold(px), TextColor(if on { ACCENT } else { FAINT })));
                        });
                    }
                    None if !big => {
                        line.spawn((Text::new("›"), cx.bold(px), TextColor(if on { ACCENT } else { FAINT })));
                    }
                    None => {}
                }
            });
            if on && !r.sub.is_empty() {
                row.spawn((Text::new(r.sub.clone()), cx.regular(19.0), TextColor(DIM)));
            }
        });
    }
}

/// The online panel: how many are on the server, and what they are doing.
fn online_panel(c: &mut Kids, cx: &Ctx, online: &OnlineInfo) {
    c.spawn((
        Node {
            width: cx.px(430.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(cx.px(24.0)),
            row_gap: cx.px(6.0),
            border: UiRect::top(cx.px(3.0)),
            ..abs(None, Some(176.0), Some(64.0), None, cx)
        },
        BackgroundColor(PANEL),
        BorderColor::all(ACCENT),
    ))
    .with_children(|p| {
        let (dot, label) = match online.status {
            Status::Online => (GREEN, "ONLINE"),
            Status::Connecting => (GOLD, "CONNECTING"),
            Status::Unreachable => (ENEMY, "UNREACHABLE"),
            Status::Offline => (FAINT, "OFFLINE"),
        };
        p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() })
            .with_children(|top| {
                top.spawn((Text::new("PLAYERS ONLINE"), cx.bold(20.0), TextColor(DIM)));
                top.spawn(Node { column_gap: cx.px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|s| {
                    s.spawn((
                        Node { width: cx.px(10.0), height: cx.px(10.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() },
                        BackgroundColor(dot),
                    ));
                    s.spawn((Text::new(label), cx.bold(18.0), TextColor(dot)));
                });
            });
        match online.status {
            Status::Online => {
                p.spawn((Text::new(fmt_count(online.players)), cx.bold(104.0), TextColor(WHITE)));
            }
            _ => {
                p.spawn((Text::new("LOCAL"), cx.bold(84.0), TextColor(TEXT.with_alpha(0.5))));
            }
        }
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), margin: UiRect::vertical(cx.px(8.0)), ..default() }, BackgroundColor(LINE)));
        let rows: Vec<(&str, String)> = match online.status {
            Status::Online => vec![
                ("IN LOBBIES", fmt_count(online.in_lobbies)),
                ("IN MATCHES", fmt_count(online.in_matches)),
                ("OPEN LOBBIES", fmt_count(online.open_lobbies)),
                ("SERVER", online.server.clone()),
            ],
            Status::Connecting => vec![("SERVER", online.server.clone()), ("STATUS", "Reaching the master server...".to_owned())],
            Status::Unreachable => vec![("SERVER", online.server.clone()), ("STATUS", "No answer. Retrying.".to_owned())],
            Status::Offline => vec![("MODE", "LOCAL PLAY".to_owned()), ("BOTS", "UP TO 17".to_owned()), ("SERVER", "NOT SET".to_owned())],
        };
        for (k, v) in rows {
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|r| {
                r.spawn((Text::new(k), cx.bold(19.0), TextColor(FAINT)));
                r.spawn((Text::new(v), cx.bold(19.0), TextColor(TEXT)));
            });
        }
    });
}

/// A picture cut to a box, covering it.
fn picture_box(c: &mut Kids, cx: &mut Ctx, names: &[String], w: f32, h: f32, extra: impl Bundle) {
    let pic = cx.pic(names);
    let size = pic.as_ref().map(|p| cx.pic_size(p));
    c.spawn((
        Node { width: cx.px(w), height: cx.px(h), overflow: Overflow::clip(), ..default() },
        BackgroundColor(Color::srgb(0.05, 0.07, 0.10)),
        extra,
    ))
    .with_children(|b| {
        if let (Some(p), Some(size)) = (pic, size) {
            let s = cover(size, Vec2::new(w, h) * cx.s);
            b.spawn((
                ImageNode::new(p),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(s.x),
                    height: Val::Px(s.y),
                    left: Val::Px((w * cx.s - s.x) * 0.5),
                    top: Val::Px((h * cx.s - s.y) * 0.5),
                    ..default()
                },
            ));
        }
    });
}

fn tabs(c: &mut Kids, cx: &mut Ctx, labels: &[String], on: usize, hit: fn(usize) -> Hit) {
    c.spawn(Node {
        column_gap: cx.px(8.0),
        align_items: AlignItems::Center,
        ..abs(Some(72.0), Some(124.0), None, None, cx)
    })
    .with_children(|t| {
        hint(t, cx, "Q", "xenonbutton_lb", "");
        for (i, l) in labels.iter().enumerate() {
            let sel = i == on;
            t.spawn((
                hit(i),
                Button,
                Node {
                    padding: UiRect::axes(cx.px(22.0), cx.px(6.0)),
                    border: UiRect::bottom(cx.px(3.0)),
                    ..default()
                },
                BorderColor::all(if sel { ACCENT } else { Color::NONE }),
                BackgroundColor(if sel { ACCENT_SOFT.with_alpha(0.16) } else { Color::NONE }),
            ))
            .with_child((Text::new(l.clone()), cx.bold(22.0), TextColor(if sel { WHITE } else { DIM })));
        }
        hint(t, cx, "E", "xenonbutton_rb", "");
    });
}

fn lobby(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    header(c, cx, &["CUSTOM GAME", "LOBBY"]);
    let labels: Vec<String> = Tab::ALL.iter().map(|t| t.label().to_owned()).collect();
    let on = Tab::ALL.iter().position(|t| *t == shell.tab).unwrap_or(0);
    tabs(c, cx, &labels, on, Hit::Tab);
    let map = cat.map(&shell.map);
    let map_title = map.map_or_else(|| shell.map.to_uppercase(), |m| m.title.clone());
    let mode_title = cat.mode(&shell.mode).map_or_else(|| shell.mode.to_uppercase(), |m| m.title.clone());
    let pictures = map.map(|m| m.pictures.clone()).unwrap_or_default();
    let starting = bridge::starting_in();
    // The left column: the match card, then the tab's rows.
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(196.0), None, None, cx)
    })
    .with_children(|col| {
        col.spawn((
            Node { width: cx.px(600.0), flex_direction: FlexDirection::Column, border: UiRect::left(cx.px(4.0)), margin: UiRect::bottom(cx.px(14.0)), ..default() },
            BorderColor::all(ACCENT),
            BackgroundColor(PANEL),
        ))
        .with_children(|card| {
            picture_box(card, cx, &pictures, 596.0, 236.0, ());
            card.spawn(Node { flex_direction: FlexDirection::Column, padding: UiRect::axes(cx.px(16.0), cx.px(12.0)), row_gap: cx.px(2.0), ..default() })
                .with_children(|t| {
                    let status = match starting {
                        Some(ms) => format!("Match starting in {}", (ms / 1000.0).ceil() as i32),
                        None => "Ready".to_owned(),
                    };
                    t.spawn((Text::new(status), cx.bold(28.0), TextColor(if starting.is_some() { GOLD } else { WHITE })));
                    t.spawn((Text::new(format!("{mode_title} on {map_title}")), cx.regular(21.0), TextColor(DIM)));
                });
        });
        menu_rows(col, cx, rows, shell.focus, 600.0, false);
        if shell.tab == Tab::Loadouts {
            col.spawn((Text::new("These screens are still Black Ops II's for now."), cx.regular(18.0), TextColor(FAINT)));
        }
    });
    // The right column: who is in.
    let total = 1 + shell.friends + shell.enemies;
    c.spawn(Node {
        width: cx.px(560.0),
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(4.0),
        ..abs(None, Some(176.0), Some(64.0), None, cx)
    })
    .with_children(|list| {
        list.spawn((Text::new(format!("{total} Players (18 Max)")), cx.regular(19.0), TextColor(DIM), Node { margin: UiRect::bottom(cx.px(6.0)), ..default() }));
        let me = &cat.player;
        let icon = cx.pic(&[me.rank_picture.clone()]);
        let diff = DIFFICULTY[shell.difficulty.min(3) as usize];
        let mut people: Vec<(String, String, Color, bool)> = vec![(me.level.to_string(), me.name.clone(), ACCENT, true)];
        for i in 0..shell.friends {
            people.push(("".to_owned(), format!("Bot {} ({})", i + 1, title_case(diff)), ACCENT, false));
        }
        for i in 0..shell.enemies {
            people.push(("".to_owned(), format!("Bot {} ({})", shell.friends + i + 1, title_case(diff)), ENEMY, false));
        }
        for (level, name, team, mine) in people {
            list.spawn((
                Node { height: cx.px(36.0), column_gap: cx.px(12.0), align_items: AlignItems::Center, padding: UiRect::horizontal(cx.px(10.0)), ..default() },
                BackgroundColor(if mine { Color::srgba(0.26, 0.62, 0.95, 0.14) } else { PANEL_SOFT }),
            ))
            .with_children(|r| {
                r.spawn((Node { width: cx.px(4.0), height: cx.px(22.0), ..default() }, BackgroundColor(team)));
                match (&icon, mine) {
                    (Some(h), true) => {
                        r.spawn((ImageNode::new(h.clone()), Node { width: cx.px(26.0), height: cx.px(26.0), ..default() }));
                    }
                    _ => {
                        r.spawn(Node { width: cx.px(26.0), ..default() });
                    }
                }
                r.spawn((Node { width: cx.px(40.0), ..default() },)).with_child((Text::new(level), cx.bold(20.0), TextColor(WHITE)));
                r.spawn((Node { flex_grow: 1.0, ..default() },))
                    .with_child((Text::new(name), cx.bold(21.0), TextColor(if mine { GOLD } else { TEXT })));
                r.spawn((Text::new("0"), cx.bold(20.0), TextColor(DIM)));
            });
        }
    });
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c.flat_map(char::to_lowercase)).collect())
}

fn map_picker(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog) {
    header(c, cx, &["CUSTOM GAME", "SELECT MAP"]);
    let p = pages(cat);
    let labels: Vec<String> = p.iter().map(|(t, _)| t.clone()).collect();
    let page = shell.page.min(p.len().saturating_sub(1));
    tabs(c, cx, &labels, page, Hit::Page);
    let Some((_, maps)) = p.get(page) else { return };
    let (tw, th, gap) = (280.0, 158.0, 14.0);
    c.spawn(Node {
        width: cx.px(COLUMNS as f32 * (tw + gap)),
        flex_wrap: FlexWrap::Wrap,
        column_gap: cx.px(gap),
        row_gap: cx.px(gap),
        ..abs(Some(72.0), Some(196.0), None, None, cx)
    })
    .with_children(|grid| {
        for (k, &i) in maps.iter().enumerate() {
            let m = &cat.maps[i];
            let on = k == shell.focus;
            let chosen = m.ui_name == shell.map;
            grid.spawn((
                Hit::Row(k),
                Button,
                Node { width: cx.px(tw), height: cx.px(th), border: UiRect::all(cx.px(if on { 3.0 } else { 1.0 })), ..default() },
                BorderColor::all(if on { ACCENT } else { LINE }),
            ))
            .with_children(|tile| {
                picture_box(tile, cx, &m.pictures, tw - if on { 6.0 } else { 2.0 }, th - if on { 6.0 } else { 2.0 }, ());
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
                        ColorStop::new(Color::srgba(0.0, 0.02, 0.05, 0.88), Val::Percent(100.0)),
                    ])),
                ))
                .with_children(|bar| {
                    bar.spawn((Text::new(m.title.clone()), cx.bold(21.0), TextColor(if on { WHITE } else { TEXT })));
                    if chosen {
                        bar.spawn((Text::new("●"), cx.bold(16.0), TextColor(ACCENT)));
                    }
                });
            });
        }
    });
    if let Some(&i) = maps.get(shell.focus) {
        let m = &cat.maps[i];
        c.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            ..abs(None, None, Some(72.0), Some(86.0), cx)
        })
        .with_children(|t| {
            t.spawn((Text::new(m.title.clone()), cx.bold(64.0), TextColor(WHITE)));
            t.spawn((Text::new(m.origin.to_uppercase()), cx.bold(22.0), TextColor(ACCENT)));
        });
    }
}

fn mode_picker(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    header(c, cx, &["CUSTOM GAME", "SELECT MODE"]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(6.0),
        ..abs(Some(72.0), Some(150.0), None, None, cx)
    })
    .with_children(|col| {
        let rows: Vec<Row> = rows.iter().map(|r| Row { sub: String::new(), ..r.clone() }).collect();
        menu_rows(col, cx, &rows, shell.focus, 520.0, false);
    });
    if let Some(m) = cat.modes.get(shell.focus) {
        c.spawn((
            Node {
                width: cx.px(560.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(cx.px(24.0)),
                row_gap: cx.px(10.0),
                border: UiRect::top(cx.px(3.0)),
                ..abs(None, Some(176.0), Some(64.0), None, cx)
            },
            BackgroundColor(PANEL),
            BorderColor::all(ACCENT),
        ))
        .with_children(|p| {
            p.spawn((Text::new(m.title.clone()), cx.bold(48.0), TextColor(WHITE)));
            p.spawn((Text::new(m.desc.clone()), cx.regular(23.0), TextColor(TEXT)));
            if m.key == shell.mode {
                p.spawn((Text::new("SELECTED"), cx.bold(19.0), TextColor(ACCENT)));
            }
        });
    }
}

fn main_menu(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        ..abs(Some(150.0), Some(300.0), None, None, cx)
    })
    .with_children(|col| {
        col.spawn((Text::new("BLACK OPS II"), cx.bold(104.0), TextColor(WHITE)));
        col.spawn((
            Node { column_gap: cx.px(12.0), align_items: AlignItems::Center, margin: UiRect::bottom(cx.px(56.0)), ..default() },
        ))
        .with_children(|t| {
            t.spawn((Node { width: cx.px(40.0), height: cx.px(3.0), ..default() }, BackgroundColor(ACCENT)));
            t.spawn((Text::new("CLASSIC MULTIPLAYER"), cx.bold(24.0), TextColor(ACCENT)));
        });
        menu_rows(col, cx, rows, shell.focus, 460.0, true);
    });
}

fn multiplayer(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row], online: &OnlineInfo) {
    header(c, cx, &["MULTIPLAYER"]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
    })
    .with_children(|col| menu_rows(col, cx, rows, shell.focus, 540.0, false));
    online_panel(c, cx, online);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn shell_view(
    mut commands: Commands,
    shell: Res<Shell>,
    fonts: Option<Res<ShellFonts>>,
    online: Option<Res<OnlineInfo>>,
    mut pics: ResMut<ShellPics>,
    icons: Option<Res<assets::T6HudIcons>>,
    mut images: ResMut<Assets<Image>>,
    roots: Query<Entity, With<ViewRoot>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut drawn: Local<String>,
) {
    let up = bridge::covers_lui();
    let cat = bridge::catalog();
    let (Some(fonts), Some(cat), true) = (fonts, cat, up) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        drawn.clear();
        return;
    };
    let online = online.as_deref().cloned().unwrap_or_default();
    let (area, s) = super::window_scale(&windows);
    let pad = crate::pad_glyphs::pad_active();
    let starting = bridge::starting_in().map(|ms| (ms / 1000.0).ceil() as i32);
    // (While pictures are still being read the view looks again twice a
    // second.)
    let tick = if pics.pending() { (time.elapsed_secs() * 2.0) as u64 } else { 0 };
    let key = format!(
        "{:?}|{:p}|{}|{online:?}|{area:?}|{pad}|{starting:?}|{tick}",
        *shell,
        std::sync::Arc::as_ptr(&cat),
        pics.count()
    );
    if *drawn == key && !roots.is_empty() {
        return;
    }
    *drawn = key;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let rows = rows(&shell, &cat);
    let mut cx = Ctx {
        s,
        fonts: &fonts,
        pics: &mut pics,
        icons: icons.as_deref(),
        images: &mut images,
        pad,
    };
    commands
        .spawn((
            ViewRoot,
            GlobalZIndex(10),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|c| {
            match shell.screen {
                Screen::Main => main_menu(c, &mut cx, &shell, &rows),
                Screen::Multiplayer => multiplayer(c, &mut cx, &shell, &rows, &online),
                Screen::Lobby => lobby(c, &mut cx, &shell, &cat, &rows),
                Screen::MapPick => map_picker(c, &mut cx, &shell, &cat),
                Screen::ModePick => mode_picker(c, &mut cx, &shell, &cat, &rows),
            }
            player_card(c, &mut cx, &cat.player);
            footer(c, &mut cx, &shell, &online);
            if let Some((note, _)) = &shell.note {
                c.spawn((
                    Node { padding: UiRect::axes(cx.px(22.0), cx.px(10.0)), border: UiRect::left(cx.px(3.0)), ..abs(Some(72.0), None, None, Some(84.0), &cx) },
                    BackgroundColor(PANEL),
                    BorderColor::all(GOLD),
                ))
                .with_child((Text::new(note.clone()), cx.bold(21.0), TextColor(WHITE)));
            }
        });
}
