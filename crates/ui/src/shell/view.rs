//! bo2mp shell: the screens, drawn again whenever what they show changes
//! (the screen, the row in focus, the lobby's choices, a picture coming in,
//! the online numbers, the window's size).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::bridge::{Catalog, PlayerCard};
use super::online::{OnlineInfo, Status};
use super::pics::{ShellPics, cover};
use super::theme::*;
use super::cards::{self, Kind};
use super::{Act, Hit, Row, Screen, Shell, Tab, bridge, pages, rows};

#[derive(Component)]
pub(crate) struct ViewRoot;

pub(crate) type Kids<'a> = ChildSpawnerCommands<'a>;

/// What the drawing needs besides the shell's state.
pub(crate) struct Ctx<'a> {
    pub s: f32,
    /// The screen in design units (1080 high).
    pub w: f32,
    pub h: f32,
    pub fonts: &'a ShellFonts,
    pub pics: &'a mut ShellPics,
    pub icons: Option<&'a assets::T6HudIcons>,
    pub images: &'a mut Assets<Image>,
    pub pad: bool,
    /// The screen's colours (blue, or the Zombies screens' red).
    pub pal: Palette,
    /// Calling cards and emblems are drawn still (a picker's page: only
    /// the one in focus and his card move).
    pub still: bool,
}

impl Ctx<'_> {
    pub fn px(&self, v: f32) -> Val {
        Val::Px(v * self.s)
    }

    pub fn bold(&self, px: f32) -> TextFont {
        font(&self.fonts.bold, px * self.s)
    }

    pub fn regular(&self, px: f32) -> TextFont {
        font(&self.fonts.regular, px * self.s)
    }

    pub fn light(&self, px: f32) -> TextFont {
        font(&self.fonts.light, px * self.s)
    }

    pub fn black(&self, px: f32) -> TextFont {
        font(&self.fonts.black, px * self.s)
    }

    pub fn pic(&mut self, names: &[String]) -> Option<Handle<Image>> {
        self.pics.first(names, self.icons, self.images)
    }

    pub fn pic_size(&self, h: &Handle<Image>) -> Vec2 {
        self.images.get(h).map_or(Vec2::new(16.0, 9.0), |i| i.size_f32())
    }
}

pub(crate) fn abs(left: Option<f32>, top: Option<f32>, right: Option<f32>, bottom: Option<f32>, cx: &Ctx) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: left.map_or(Val::Auto, |v| cx.px(v)),
        top: top.map_or(Val::Auto, |v| cx.px(v)),
        right: right.map_or(Val::Auto, |v| cx.px(v)),
        bottom: bottom.map_or(Val::Auto, |v| cx.px(v)),
        ..default()
    }
}

/// The title at the top left, as Infinite Warfare's: the trail of screens
/// above in small capitals (`MULTIPLAYER / CUSTOM GAME`), the screen's name
/// under it large and thin.
pub(crate) fn header(c: &mut Kids, cx: &Ctx, trail: &[&str]) {
    let (title, before) = trail.split_last().map_or(("", &[][..]), |(t, b)| (*t, b));
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(0.0), ..abs(Some(96.0), Some(30.0), None, None, cx) }).with_children(|h| {
        h.spawn((Text::new(before.join(" / ")), cx.regular(19.0), TextColor(TEXT.with_alpha(0.86)), Node { min_height: cx.px(22.0), margin: UiRect::left(cx.px(3.0)), ..default() }));
        h.spawn((Text::new(title), cx.regular(62.0), TextColor(WHITE), Node { margin: UiRect::top(cx.px(-8.0)), ..default() }));
    });
}

/// The first word of a shared screen's header (Barracks, Create-a-Class):
/// MULTIPLAYER, or WARZONE when opened from its menus.
pub(crate) fn section(shell: &Shell) -> &'static str {
    if shell.wz { super::wz_theme::HEADER } else { "MULTIPLAYER" }
}

/// A calling card: the picture covering a box, dark to its left so a name
/// over it reads (`name`: the name and its size).
pub(crate) fn banner(c: &mut Kids, cx: &mut Ctx, card: &str, w: f32, h: f32, name: Option<(&str, f32)>) {
    let pic = cx.pic(&[card.to_owned()]);
    let size = pic.as_ref().map(|p| cx.pic_size(p));
    c.spawn((Node { width: cx.px(w), height: cx.px(h), flex_shrink: 0.0, overflow: Overflow::clip(), ..default() }, BackgroundColor(cx.pal.picture_bg)))
        .with_children(|b| {
            match (pic, size) {
                (Some(p), Some(size)) => {
                    let s = cover(size, Vec2::new(w, h) * cx.s);
                    let book = flipbook(cx, card);
                    let mut e = b.spawn((
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
                    if let Some(book) = book {
                        e.insert(book);
                    }
                }
                // (Not in yet, or the game is not installed: a plain plate.)
                _ => {
                    b.spawn((
                        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                        BackgroundGradient::from(LinearGradient::to_right(vec![
                            ColorStop::new(cx.pal.accent_a(0.10), Val::Percent(0.0)),
                            ColorStop::new(cx.pal.accent_a(0.32), Val::Percent(100.0)),
                        ])),
                    ));
                }
            }
            if let Some((text, px)) = name {
                b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        padding: UiRect::horizontal(cx.px(px * 0.4)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundGradient::from(LinearGradient::to_right(vec![
                        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.62), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(85.0)),
                    ])),
                ))
                .with_child((Text::new(text), cx.bold(px), TextColor(WHITE)));
            }
        });
}

/// A flipbook's frames on screen (Modern Warfare III's animated emblems
/// and calling cards), turned by [`turn_flipbooks`].
#[derive(Component, Clone, Debug)]
pub(crate) struct Flipbook {
    pub frames: Vec<Handle<Image>>,
}

/// Frames a second a flipbook turns at.
const FLIPBOOK_FPS: f32 = 12.0;

/// The flipbook of a calling card's or emblem's picture, once its frames
/// are in (its still picture shows till then): `None` for a still one, and
/// while they are being read (asking for them; the view is drawn again as
/// they come in).
pub(crate) fn flipbook(cx: &mut Ctx, name: &str) -> Option<Flipbook> {
    let n = cards::frames(name);
    if n < 2 || cx.still {
        return None;
    }
    let mut frames = Vec::with_capacity(n as usize);
    for i in 0..n {
        frames.push(cx.pic(&[cards::frame_name(name, i)])?);
    }
    Some(Flipbook { frames })
}

/// Turns every flipbook on screen.
pub(crate) fn turn_flipbooks(time: Res<Time>, mut books: Query<(&mut ImageNode, &Flipbook)>) {
    let at = (time.elapsed_secs() * FLIPBOOK_FPS) as usize;
    for (mut img, book) in &mut books {
        if let Some(f) = book.frames.get(at % book.frames.len().max(1))
            && img.image != *f
        {
            img.image = f.clone();
        }
    }
}

/// An emblem in a square.
pub(crate) fn emblem(c: &mut Kids, cx: &mut Ctx, name: &str, size: f32) {
    let pic = cx.pic(&[name.to_owned()]);
    c.spawn((
        Node { width: cx.px(size), height: cx.px(size), flex_shrink: 0.0, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(cx.pal.panel_soft),
    ))
    .with_children(|b| match pic {
        Some(p) => {
            let sz = cx.pic_size(&p);
            let k = (size / sz.x.max(1.0)).min(size / sz.y.max(1.0));
            let book = flipbook(cx, name);
            let mut e = b.spawn((ImageNode::new(p), Node { width: cx.px(sz.x * k), height: cx.px(sz.y * k), ..default() }));
            if let Some(book) = book {
                e.insert(book);
            }
        }
        None => {
            b.spawn((
                Node { width: cx.px(size * 0.42), height: cx.px(size * 0.42), border: UiRect::all(cx.px(2.0)), ..default() },
                BorderColor::all(cx.pal.accent_a(0.45)),
                UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
            ));
        }
    });
}

/// His card at the top right, as Infinite Warfare's: a line of his numbers
/// (rank and level, prestige, kills, wins) over a thin rule, then his
/// calling card as a banner with his name and level on it, a band in the
/// accent with his progress, and his emblem standing over its right end.
fn player_card(c: &mut Kids, cx: &mut Ctx, card: &PlayerCard) {
    let icon = cx.pic(&[card.rank_picture.clone()]);
    let (title, mark) = (cards::mine(Kind::Card, &card.card), cards::mine(Kind::Emblem, &card.emblem));
    let w = 640.0;
    c.spawn(Node { width: cx.px(w + 90.0), flex_direction: FlexDirection::Column, ..abs(None, Some(26.0), Some(0.0), None, cx) }).with_children(|p| {
        // The numbers.
        p.spawn((
            Node { width: cx.px(w), height: cx.px(34.0), column_gap: cx.px(28.0), align_items: AlignItems::Center, padding: UiRect::horizontal(cx.px(10.0)), border: UiRect::bottom(cx.px(2.0)), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            BorderColor::all(WHITE.with_alpha(0.85)),
        ))
        .with_children(|n| {
            n.spawn(Node { column_gap: cx.px(6.0), align_items: AlignItems::Center, ..default() }).with_children(|r| {
                if let Some(h) = icon.clone() {
                    r.spawn((ImageNode::new(h), Node { width: cx.px(24.0), height: cx.px(24.0), ..default() }));
                }
                r.spawn((Text::new(card.level.to_string()), cx.bold(22.0), TextColor(WHITE)));
            });
            if card.prestige > 0 {
                n.spawn((Text::new(format!("PRESTIGE {}", card.prestige)), cx.bold(20.0), TextColor(TEXT)));
            }
            for (k, v) in card.record.iter().take(3) {
                n.spawn(Node { column_gap: cx.px(6.0), align_items: AlignItems::Baseline, ..default() }).with_children(|r| {
                    r.spawn((Text::new(v.clone()), cx.bold(21.0), TextColor(WHITE)));
                    r.spawn((Text::new(k.clone()), cx.regular(16.0), TextColor(DIM)));
                });
            }
        });
        // The banner, the band, the emblem over its end.
        p.spawn(Node { width: cx.px(w + 90.0), height: cx.px(112.0), ..default() }).with_children(|b| {
            b.spawn((Node { width: cx.px(w), height: cx.px(112.0), flex_direction: FlexDirection::Column, border: UiRect::all(cx.px(1.0)), ..default() }, BorderColor::all(cx.pal.accent_a(0.65))))
                .with_children(|card_box| {
                    card_box.spawn(Node { width: Val::Percent(100.0), height: cx.px(62.0), ..default() }).with_children(|top| {
                        banner(top, cx, &title, w - 2.0, 62.0, None);
                        top.spawn(Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: cx.px(120.0),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::Center,
                            column_gap: cx.px(28.0),
                            ..default()
                        })
                        .with_children(|t| {
                            t.spawn((Text::new(card.name.to_uppercase()), cx.bold(34.0), TextColor(WHITE), TextShadow { offset: Vec2::splat(2.0 * cx.s), color: Color::srgba(0.0, 0.0, 0.0, 0.8), ..default() }));
                            t.spawn((Text::new(card.level.to_string()), cx.bold(34.0), TextColor(WHITE), TextShadow { offset: Vec2::splat(2.0 * cx.s), color: Color::srgba(0.0, 0.0, 0.0, 0.8), ..default() }));
                        });
                    });
                    card_box
                        .spawn((
                            Node { width: Val::Percent(100.0), flex_grow: 1.0, flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, row_gap: cx.px(5.0), padding: UiRect::horizontal(cx.px(10.0)), ..default() },
                            BackgroundGradient::from(LinearGradient::to_right(vec![
                                ColorStop::new(cx.pal.accent_a(0.85), Val::Percent(0.0)),
                                ColorStop::new(cx.pal.accent_a(0.35), Val::Percent(70.0)),
                                ColorStop::new(cx.pal.accent_a(0.15), Val::Percent(100.0)),
                            ])),
                        ))
                        .with_children(|band| {
                            let prestige = if card.prestige > 0 { format!("PRESTIGE {}  ·  ", card.prestige) } else { String::new() };
                            band.spawn((Text::new(format!("{prestige}LEVEL {}  ·  {} XP", card.level, card.xp)), cx.bold(18.0), TextColor(WHITE)));
                            band.spawn((Node { width: cx.px(w * 0.62), height: cx.px(4.0), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45))))
                                .with_child((Node { width: Val::Percent(card.xp_frac * 100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(WHITE)));
                        });
                });
            b.spawn(Node { position_type: PositionType::Absolute, right: cx.px(6.0), top: cx.px(-22.0), ..default() }).with_children(|e| {
                emblem_bare(e, cx, &mark, 132.0);
            });
        });
    });
}

/// An emblem with no plate behind it (the player card's, standing over the
/// banner's end).
pub(crate) fn emblem_bare(c: &mut Kids, cx: &mut Ctx, name: &str, size: f32) {
    let pic = cx.pic(&[name.to_owned()]);
    c.spawn(Node { width: cx.px(size), height: cx.px(size), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }).with_children(|b| {
        if let Some(p) = pic {
            let sz = cx.pic_size(&p);
            let k = (size / sz.x.max(1.0)).min(size / sz.y.max(1.0));
            let book = flipbook(cx, name);
            let mut e = b.spawn((ImageNode::new(p), Node { width: cx.px(sz.x * k), height: cx.px(sz.y * k), ..default() }));
            if let Some(book) = book {
                e.insert(book);
            }
        }
    });
}

/// A key hint: the key (or the pad's button picture) and what it does.
pub(crate) fn hint(c: &mut Kids, cx: &mut Ctx, key: &str, pad: &str, what: &str) {
    let icon = if cx.pad && !pad.is_empty() { cx.pic(&[pad.to_owned()]) } else { None };
    c.spawn(Node { column_gap: cx.px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|h| {
        match icon {
            Some(img) => {
                h.spawn((ImageNode::new(img), Node { width: cx.px(30.0), height: cx.px(30.0), ..default() }));
            }
            None if !key.is_empty() => {
                h.spawn((
                    Node { min_width: cx.px(30.0), height: cx.px(30.0), padding: UiRect::horizontal(cx.px(8.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(cx.px(2.0)), border_radius: BorderRadius::all(cx.px(15.0)), ..default() },
                    BorderColor::all(WHITE.with_alpha(0.85)),
                ))
                .with_child((Text::new(key), cx.bold(16.0), TextColor(WHITE)));
            }
            None => {}
        }
        h.spawn((Text::new(what), cx.regular(24.0), TextColor(TEXT.with_alpha(0.86))));
    });
}

fn footer(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, online: &OnlineInfo) {
    // Infinite Warfare's foot: a thin lit band across the left of the
    // screen, the keys under it on black, the server and friends at the right.
    c.spawn((
        Node { width: Val::Percent(62.0), height: cx.px(5.0), ..abs(Some(0.0), None, None, Some(66.0), cx) },
        BackgroundGradient::from(LinearGradient::to_right(vec![
            ColorStop::new(Color::srgba(0.55, 0.58, 0.62, 0.55), Val::Percent(0.0)),
            ColorStop::new(cx.pal.accent_a(0.55), Val::Percent(70.0)),
            ColorStop::new(Color::srgba(0.55, 0.58, 0.62, 0.0), Val::Percent(100.0)),
        ])),
    ));
    c.spawn((
        Node {
            width: Val::Percent(100.0),
            height: cx.px(66.0),
            padding: UiRect::horizontal(cx.px(108.0)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..abs(Some(0.0), None, None, Some(0.0), cx)
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.70), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.92), Val::Percent(100.0)),
        ])),
    ))
    .with_children(|f| {
        f.spawn((Node { column_gap: cx.px(28.0), align_items: AlignItems::Center, ..default() }, Hit::Back, Button))
            .with_children(|l| {
                if shell.screen != Screen::Main {
                    let back = match shell.screen {
                        Screen::Lobby => "LEAVE LOBBY",
                        Screen::WzLobby => super::wz_theme::LEAVE_LOBBY,
                        _ => "BACK",
                    };
                    hint(l, cx, "ESC", "xenonbutton_b", back);
                }
                hint(l, cx, "ENTER", "xenonbutton_a", "SELECT");
                match shell.screen {
                    Screen::Lobby | Screen::WzLobby | Screen::Options | Screen::MatchFilter => hint(l, cx, "Q / E", "xenonbutton_lb", "TABS"),
                    Screen::MapPick => hint(l, cx, "Q / E", "xenonbutton_lb", "SECTIONS"),
                    Screen::ItemPick => hint(l, cx, "Q / E", "xenonbutton_lb", "CATEGORIES"),
                    Screen::CardPick | Screen::EmblemPick => hint(l, cx, "Q / E", "xenonbutton_lb", "PAGES"),
                    Screen::ZmMapPick if super::zombies::pages(super::zombies::state().bo1()).len() > 1 => hint(l, cx, "Q / E", "xenonbutton_lb", "SECTIONS"),
                    _ => {}
                }
                let stepped = super::rows(shell, &cat).get(shell.focus).is_some_and(|r| matches!(r.act, Act::Cycle(_)));
                if stepped || shell.screen == Screen::ClassEdit {
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

/// A column of menu rows, as Infinite Warfare's: each a white bar at its
/// left and its name; the one in focus a band in the accent ending in a
/// point. A selector's value sits at the row's right between arrows. The
/// line about the row in focus goes under the column. `big`: taller rows.
pub(crate) fn menu_rows(c: &mut Kids, cx: &Ctx, rows: &[Row], focus: usize, width: f32, big: bool) {
    let (h, px) = if big { (50.0, 30.0) } else { (42.0, 25.0) };
    let tip = h * 0.5;
    for (i, r) in rows.iter().enumerate() {
        let on = i == focus;
        c.spawn((
            Hit::Row(i),
            Button,
            Node { width: cx.px(width + tip), min_height: cx.px(h), flex_shrink: 0.0, ..default() },
        ))
        .with_children(|row| {
            let mut band = row.spawn(Node {
                width: cx.px(width),
                min_height: cx.px(h),
                padding: UiRect::new(cx.px(18.0), cx.px(16.0), cx.px(3.0), cx.px(3.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::left(cx.px(6.0)),
                ..default()
            });
            if on {
                band.insert((
                    BorderColor::all(cx.pal.accent),
                    BackgroundGradient::from(LinearGradient::to_right(vec![
                        ColorStop::new(cx.pal.accent, Val::Percent(0.0)),
                        ColorStop::new(lift(cx.pal.accent, 0.10), Val::Percent(100.0)),
                    ])),
                ));
            } else {
                band.insert((BorderColor::all(WHITE.with_alpha(0.88)), BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.42))));
            }
            band.with_children(|line| {
                let ink = if on { WHITE } else { TEXT.with_alpha(0.86) };
                line.spawn((Text::new(r.label.clone()), if on { cx.bold(px) } else { cx.regular(px) }, TextColor(ink)));
                match &r.value {
                    // (A value typed in, not stepped: no arrows.)
                    Some(v) if !matches!(r.act, Act::Cycle(_)) => {
                        line.spawn((Text::new(v.clone()), cx.bold(px * 0.9), TextColor(if on { WHITE } else { DIM })));
                    }
                    Some(v) => {
                        line.spawn(Node { column_gap: cx.px(12.0), align_items: AlignItems::Center, ..default() }).with_children(|sel| {
                            sel.spawn((Hit::Step(i, -1), Button, Text::new("‹"), cx.bold(px), TextColor(if on { WHITE } else { FAINT })));
                            sel.spawn(Node { min_width: cx.px(160.0), justify_content: JustifyContent::Center, ..default() })
                                .with_child((Text::new(v.clone()), cx.bold(px * 0.9), TextColor(if on { WHITE } else { DIM })));
                            sel.spawn((Hit::Step(i, 1), Button, Text::new("›"), cx.bold(px), TextColor(if on { WHITE } else { FAINT })));
                        });
                    }
                    None => {}
                }
            });
            if on {
                // The point: a square turned on its corner, half out past
                // the band's end.
                row.spawn(Node { width: cx.px(tip), height: cx.px(h), overflow: Overflow::clip(), ..default() }).with_child((
                    Node { position_type: PositionType::Absolute, left: cx.px(-h * 0.355), top: cx.px(h * 0.145), width: cx.px(h * 0.71), height: cx.px(h * 0.71), ..default() },
                    BackgroundColor(lift(cx.pal.accent, 0.10)),
                    UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
                ));
            }
        });
    }
    if let Some(r) = rows.get(focus).filter(|r| !r.sub.is_empty()) {
        c.spawn((Text::new(r.sub.clone()), cx.regular(23.0), TextColor(WHITE), Node { max_width: cx.px(width + 40.0), margin: UiRect::top(cx.px(10.0)), ..default() }));
    }
}

/// A colour a little lighter.
pub(crate) fn lift(c: Color, by: f32) -> Color {
    let l = c.to_srgba();
    Color::srgba((l.red + by).min(1.0), (l.green + by).min(1.0), (l.blue + by).min(1.0), l.alpha)
}

/// A picture cut to a box, covering it.
pub(crate) fn picture_box(c: &mut Kids, cx: &mut Ctx, names: &[String], w: f32, h: f32, extra: impl Bundle) {
    let pic = cx.pic(names);
    let size = pic.as_ref().map(|p| cx.pic_size(p));
    c.spawn((
        Node { width: cx.px(w), height: cx.px(h), overflow: Overflow::clip(), ..default() },
        BackgroundColor(cx.pal.picture_bg),
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

/// A picture fitted inside a box, keeping its shape.
pub(crate) fn picture_contain(c: &mut Kids, cx: &mut Ctx, names: &[String], w: f32, h: f32) {
    let pic = cx.pic(names);
    c.spawn((Node { width: cx.px(w), height: cx.px(h), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },))
        .with_children(|b| {
            if let Some(p) = pic {
                let size = cx.pic_size(&p);
                let k = (w / size.x.max(1.0)).min(h / size.y.max(1.0));
                // (A weapon's menu picture is drawn mirrored, barrel to the
                // left, as Black Ops II's Create-a-Class shows it.)
                let weapon = names.first().is_some_and(|n| n.starts_with("menu_mp_weapons") || n.contains("_weapons_"));
                let mut img = ImageNode::new(p);
                img.flip_x = weapon;
                b.spawn((img, Node { width: cx.px(size.x * k), height: cx.px(size.y * k), ..default() }));
            }
        });
}

pub(crate) fn tabs(c: &mut Kids, cx: &mut Ctx, labels: &[String], on: usize, hit: fn(usize) -> Hit) {
    c.spawn(Node {
        column_gap: cx.px(8.0),
        align_items: AlignItems::Center,
        ..abs(Some(96.0), Some(132.0), None, None, cx)
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
                BorderColor::all(if sel { cx.pal.accent } else { Color::NONE }),
                BackgroundColor(if sel { cx.pal.accent_soft.with_alpha(0.16) } else { Color::NONE }),
            ))
            .with_child((Text::new(l.clone()), cx.bold(22.0), TextColor(if sel { WHITE } else { DIM })));
        }
        hint(t, cx, "E", "xenonbutton_rb", "");
    });
}

/// A small white point to the right (his own line in a list).
pub(crate) fn pointer(c: &mut Kids, cx: &Ctx) {
    c.spawn(Node { width: cx.px(10.0), height: cx.px(16.0), overflow: Overflow::clip(), ..default() }).with_child((
        Node { position_type: PositionType::Absolute, left: cx.px(-6.0), top: cx.px(2.3), width: cx.px(11.3), height: cx.px(11.3), ..default() },
        BackgroundColor(WHITE),
        UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
    ));
}

/// A player in a lobby's list.
pub(crate) struct LobbyPlayer {
    pub level: Option<i32>,
    pub prestige: i32,
    pub name: String,
    pub me: bool,
    pub host: bool,
    /// His team's colour (a bar at the row's left), if teams are known.
    pub team: Option<Color>,
    /// His calling card (drawn as the banner at the row's right).
    pub card: String,
}

/// The lobby's players, as Infinite Warfare's: a line each, his rank and
/// level, his name, and his calling card as a banner at the right; empty
/// places a faint line. `title`: the line over the list (players / max).
pub(crate) fn player_list(c: &mut Kids, cx: &mut Ctx, players: &[LobbyPlayer], max: usize, title: &str, top: f32) {
    c.spawn(Node { width: cx.px(640.0), flex_direction: FlexDirection::Column, ..abs(None, Some(top), Some(90.0), None, cx) }).with_children(|list| {
        list.spawn(Node { justify_content: JustifyContent::SpaceBetween, margin: UiRect::bottom(cx.px(6.0)), ..default() }).with_children(|t| {
            t.spawn((Text::new(title.to_owned()), cx.regular(24.0), TextColor(WHITE)));
            t.spawn((Text::new(format!("{}/{max}", players.len().min(max))), cx.regular(24.0), TextColor(WHITE)));
        });
        for i in 0..max.max(players.len()) {
            let p = players.get(i);
            list.spawn((
                Node { height: cx.px(40.0), column_gap: cx.px(12.0), align_items: AlignItems::Center, border: UiRect::bottom(cx.px(1.0)), flex_shrink: 0.0, ..default() },
                BorderColor::all(WHITE.with_alpha(if p.is_some() { 0.30 } else { 0.10 })),
                BackgroundGradient::from(LinearGradient::to_right(vec![
                    ColorStop::new(Color::srgba(0.0, 0.0, 0.0, if p.is_some_and(|p| p.me) { 0.62 } else { 0.48 }), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.12), Val::Percent(100.0)),
                ])),
            ))
            .with_children(|r| {
                let Some(p) = p else {
                    r.spawn((Text::new("—"), cx.regular(20.0), TextColor(FAINT), Node { margin: UiRect::left(cx.px(30.0)), ..default() }));
                    return;
                };
                r.spawn((Node { width: cx.px(4.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(p.team.unwrap_or(Color::NONE))));
                r.spawn(Node { width: cx.px(14.0), ..default() }).with_children(|m| {
                    if p.me {
                        pointer(m, cx);
                    }
                });
                let icon = p.level.and_then(|l| cx.pic(&[bo2_profile::progression::rank_picture(l, p.prestige)]));
                match icon {
                    Some(h) => {
                        r.spawn((ImageNode::new(h), Node { width: cx.px(28.0), height: cx.px(28.0), ..default() }));
                    }
                    None => {
                        r.spawn(Node { width: cx.px(28.0), ..default() });
                    }
                }
                r.spawn((Node { width: cx.px(46.0), ..default() },)).with_child((Text::new(p.level.map_or_else(String::new, |l| l.to_string())), cx.regular(23.0), TextColor(WHITE)));
                r.spawn((Node { flex_grow: 1.0, column_gap: cx.px(10.0), align_items: AlignItems::Center, ..default() },)).with_children(|n| {
                    n.spawn((Text::new(p.name.clone()), cx.regular(23.0), TextColor(if p.me { GOLD } else { WHITE })));
                    if p.host {
                        n.spawn((Text::new("HOST"), cx.bold(16.0), TextColor(cx.pal.accent)));
                    }
                });
                banner(r, cx, &p.card, 176.0, 36.0, None);
            });
        }
    });
}

/// A map in the lobby's panel at the bottom left, as Infinite Warfare's vote
/// rows: its picture behind, a diamond mark, its name and mode, and a figure
/// at the right (votes, or the countdown).
#[allow(clippy::too_many_arguments)]
pub(crate) fn map_tile(c: &mut Kids, cx: &mut Ctx, pictures: &[String], name: &str, mode: &str, right: (&str, &str), on: bool, hit: Option<Hit>) {
    let (w, h) = (560.0, 86.0);
    let mut e = c.spawn((
        Node { width: cx.px(w), height: cx.px(h), border: UiRect::all(cx.px(if on { 2.0 } else { 1.0 })), flex_shrink: 0.0, ..default() },
        BorderColor::all(if on { WHITE } else { WHITE.with_alpha(0.25) }),
    ));
    if let Some(hit) = hit {
        e.insert((hit, Button));
    }
    e.with_children(|tile| {
        picture_box(tile, cx, pictures, w - if on { 4.0 } else { 2.0 }, h - if on { 4.0 } else { 2.0 }, ());
        tile.spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), right: Val::Px(0.0), align_items: AlignItems::Center, column_gap: cx.px(16.0), padding: UiRect::horizontal(cx.px(18.0)), ..default() },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::new(if on { cx.pal.accent_a(0.70) } else { Color::srgba(0.0, 0.0, 0.0, 0.72) }, Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.35), Val::Percent(62.0)),
                ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.70), Val::Percent(100.0)),
            ])),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node { width: cx.px(46.0), height: cx.px(46.0), border: UiRect::all(cx.px(3.0)), margin: UiRect::horizontal(cx.px(8.0)), ..default() },
                BorderColor::all(WHITE.with_alpha(0.9)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
            ));
            bar.spawn(Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, ..default() }).with_children(|t| {
                t.spawn((Text::new(name.to_owned()), cx.bold(28.0), TextColor(WHITE)));
                t.spawn((Text::new(mode.to_owned()), cx.bold(18.0), TextColor(TEXT.with_alpha(0.8))));
            });
            bar.spawn(Node { column_gap: cx.px(8.0), align_items: AlignItems::Baseline, ..default() }).with_children(|t| {
                t.spawn((Text::new(right.0.to_owned()), cx.bold(20.0), TextColor(WHITE)));
                t.spawn((Text::new(right.1.to_owned()), cx.bold(26.0), TextColor(GOLD)));
            });
        });
    });
}

/// The lobby's map, as Infinite Warfare's pre-match lobby shows it at the
/// bottom left: the map's name over a framed picture of it, the mode's mark
/// and name across the picture's foot; what the lobby is doing at the right
/// of the name (the countdown, the players it waits for).
pub(crate) fn map_card(c: &mut Kids, cx: &mut Ctx, pictures: &[String], map: &str, mode: &str, status: &str, gold: bool) {
    let (w, h) = (470.0, 250.0);
    c.spawn(Node { width: cx.px(w), flex_direction: FlexDirection::Column, row_gap: cx.px(4.0), ..abs(Some(126.0), None, None, Some(96.0), cx) }).with_children(|col| {
        col.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() }).with_children(|t| {
            t.spawn(Node { column_gap: cx.px(8.0), align_items: AlignItems::Center, ..default() }).with_children(|n| {
                n.spawn((Node { width: cx.px(6.0), height: cx.px(6.0), ..default() }, BackgroundColor(WHITE)));
                n.spawn((Text::new(title_word(map)), cx.regular(22.0), TextColor(WHITE)));
            });
            t.spawn((Text::new(status.to_owned()), cx.regular(20.0), TextColor(if gold { GOLD } else { TEXT })));
        });
        col.spawn((Node { width: cx.px(w), height: cx.px(h), border: UiRect::all(cx.px(2.0)), ..default() }, BorderColor::all(WHITE.with_alpha(0.9)))).with_children(|f| {
            picture_box(f, cx, pictures, w - 4.0, h - 4.0, ());
            f.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), bottom: Val::Px(0.0), height: cx.px(92.0), align_items: AlignItems::Center, column_gap: cx.px(18.0), padding: UiRect::horizontal(cx.px(22.0)), ..default() },
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.82), Val::Percent(100.0)),
                ])),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Node { width: cx.px(44.0), height: cx.px(44.0), border: UiRect::all(cx.px(3.0)), margin: UiRect::horizontal(cx.px(6.0)), ..default() },
                    BorderColor::all(WHITE),
                    BackgroundColor(cx.pal.accent_a(0.55)),
                    UiTransform { rotation: Rot2::degrees(45.0), ..UiTransform::IDENTITY },
                ));
                bar.spawn((Text::new(mode.to_uppercase()), cx.bold(34.0), TextColor(WHITE), TextShadow { offset: Vec2::splat(2.0 * cx.s), color: Color::srgba(0.0, 0.0, 0.0, 0.8), ..default() }));
            });
        });
    });
}

/// `NUKETOWN 2025` -> `Nuketown 2025` (the card's small map name).
fn title_word(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c.flat_map(char::to_lowercase)).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The private lobby (Custom Game), as Infinite Warfare's pre-match lobby:
/// the mode as the title, its tabs, the rows at the left and the map under
/// them, the players at the right.
fn lobby(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    let map = cat.map(&shell.map);
    let map_title = map.map_or_else(|| shell.map.to_uppercase(), |m| m.title.clone());
    let mode_title = cat.mode(&shell.mode).map_or_else(|| shell.mode.to_uppercase(), |m| m.title.clone());
    let pictures = map.map(|m| m.pictures.clone()).unwrap_or_default();
    header(c, cx, &["MULTIPLAYER", "CUSTOM GAME", &mode_title]);
    let labels: Vec<String> = Tab::ALL.iter().map(|t| t.label().to_owned()).collect();
    let on = Tab::ALL.iter().position(|t| *t == shell.tab).unwrap_or(0);
    tabs(c, cx, &labels, on, Hit::Tab);
    let starting = bridge::starting_in();
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(6.0), ..abs(Some(126.0), Some(196.0), None, None, cx) })
        .with_children(|col| menu_rows(col, cx, rows, shell.focus, 470.0, false));
    let when = starting.map_or_else(|| format!("{} BOTS  ·  READY", shell.friends + shell.enemies), |ms| format!("STARTING IN 0:{:02}", (ms / 1000.0).ceil() as i32));
    map_card(c, cx, &pictures, &map_title, &mode_title, &when, starting.is_some());
    let me = &cat.player;
    let mut people = vec![LobbyPlayer {
        level: Some(me.level),
        prestige: me.prestige,
        name: me.name.clone(),
        me: true,
        host: true,
        team: Some(cx.pal.accent),
        card: cards::mine(Kind::Card, &me.card),
    }];
    // bo2mp: each bot by the name and level it plays the match under.
    for i in 0..shell.friends + shell.enemies {
        let n = i + 1;
        let name = sim::lobby_bots::lobby_bot_name(n);
        people.push(LobbyPlayer {
            level: Some(sim::lobby_bots::level_for(&name)),
            prestige: 0,
            card: cards::for_name(Kind::Card, &format!("Bot {n}")),
            name,
            me: false,
            host: false,
            team: Some(if i < shell.friends { cx.pal.accent } else { ENEMY }),
        });
    }
    player_list(c, cx, &people, 18, "LOBBY", 196.0);
}

/// A grid that fits the screen: how many columns fit, and which tiles
/// (first..last) show so the one in focus is always in view. Input moves
/// by the columns drawn (`GRID_COLS`).
pub(crate) fn grid_window(cx: &Ctx, n: usize, focus: usize, tw: f32, th: f32, gap: f32, top: f32, bottom: f32) -> (usize, usize, usize) {
    let cols = (((cx.w - 144.0 + gap) / (tw + gap)).floor() as usize).clamp(2, super::COLUMNS);
    let rows = (((cx.h - top - bottom + gap) / (th + gap)).floor() as usize).max(1);
    super::GRID_COLS.store(cols, std::sync::atomic::Ordering::Relaxed);
    let focus_row = focus / cols;
    let total_rows = n.div_ceil(cols).max(1);
    let mut first_row = super::GRID_FIRST_ROW.load(std::sync::atomic::Ordering::Relaxed);
    if focus_row < first_row {
        first_row = focus_row;
    }
    if focus_row >= first_row + rows {
        first_row = focus_row + 1 - rows;
    }
    first_row = first_row.min(total_rows.saturating_sub(rows));
    super::GRID_FIRST_ROW.store(first_row, std::sync::atomic::Ordering::Relaxed);
    (cols, first_row * cols, ((first_row + rows) * cols).min(n))
}

/// "12-24 of 30", and arrows when more lie above or below.
pub(crate) fn grid_more(c: &mut Kids, cx: &Ctx, first: usize, last: usize, n: usize, left: f32, top: f32) {
    if first == 0 && last >= n {
        return;
    }
    let up = if first > 0 { "▲ " } else { "" };
    let down = if last < n { " ▼" } else { "" };
    c.spawn((
        Node { position_type: PositionType::Absolute, left: cx.px(left), top: cx.px(top), ..default() },
        Text::new(format!("{up}{}-{} OF {n}{down}", first + 1, last)),
        cx.bold(19.0),
        TextColor(DIM),
    ));
}

fn map_picker(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog) {
    let p = pages(cat);
    let page = shell.page.min(p.len().saturating_sub(1));
    let section = p.get(page).map_or(String::new(), |(t, _)| t.clone());
    header(c, cx, &["CUSTOM GAME", "SELECT MAP", section.as_str()]);
    // (Short names, so every game's tab fits on one line.)
    let labels: Vec<String> = p.iter().map(|(t, _)| short_section(t)).collect();
    tabs(c, cx, &labels, page, Hit::Page);
    let Some((_, maps)) = p.get(page) else { return };
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
            let m = &cat.maps[i];
            let on = k == shell.focus;
            let chosen = m.ui_name == shell.map;
            grid.spawn((
                Hit::Row(k),
                Button,
                Node { width: cx.px(tw), height: cx.px(th), border: UiRect::all(cx.px(if on { 3.0 } else { 1.0 })), ..default() },
                BorderColor::all(if on { cx.pal.accent } else { cx.pal.line }),
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
                        ColorStop::new(cx.pal.strip, Val::Percent(100.0)),
                    ])),
                ))
                .with_children(|bar| {
                    bar.spawn((Text::new(m.title.clone()), cx.bold(21.0), TextColor(if on { WHITE } else { TEXT })));
                    if chosen {
                        bar.spawn((Text::new("●"), cx.bold(16.0), TextColor(cx.pal.accent)));
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
            t.spawn((Text::new(m.origin.to_uppercase()), cx.bold(22.0), TextColor(cx.pal.accent)));
        });
    }
}

/// A map section's tab name (`MODERN WARFARE 3 2` -> `MW3 2`).
pub(crate) fn short_section(title: &str) -> String {
    const SHORT: [(&str, &str); 7] = [
        ("BLACK OPS II", "BO2"),
        ("BLACK OPS", "BO1"),
        ("MODERN WARFARE 3", "MW3"),
        ("MODERN WARFARE 2", "MW2"),
        ("WORLD AT WAR", "WAW"),
        ("CALL OF DUTY 4", "COD4"),
        ("CUSTOM MAPS", "CUSTOM"),
    ];
    SHORT
        .iter()
        .find_map(|(long, short)| title.strip_prefix(long).map(|rest| format!("{short}{rest}")))
        .unwrap_or_else(|| title.to_owned())
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
            BackgroundColor(cx.pal.panel),
            BorderColor::all(cx.pal.accent),
        ))
        .with_children(|p| {
            p.spawn((Text::new(m.title.clone()), cx.bold(48.0), TextColor(WHITE)));
            p.spawn((Text::new(m.desc.clone()), cx.regular(23.0), TextColor(TEXT)));
            if m.key == shell.mode {
                p.spawn((Text::new("SELECTED"), cx.bold(19.0), TextColor(cx.pal.accent)));
            }
        });
    }
}

/// Options: the tabs, the rows of the one he is on, and a line about the
/// row in focus.
pub(crate) fn options_view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    header(c, cx, &["OPTIONS"]);
    let labels: Vec<String> = super::options::TABS.iter().map(|t| (*t).to_owned()).collect();
    tabs(c, cx, &labels, shell.page.min(labels.len() - 1), Hit::Page);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(6.0),
        ..abs(Some(72.0), Some(196.0), None, None, cx)
    })
    .with_children(|col| {
        let shown: Vec<Row> = rows.iter().map(|r| Row { sub: String::new(), ..r.clone() }).collect();
        menu_rows(col, cx, &shown, shell.focus, 760.0, false);
    });
    // bo2mp: the CONTROLLER tab draws the pad and what each button does.
    let controller = shell.page == 3;
    if controller {
        super::pad_diagram::view(c, cx, &super::options::snapshot(), 312.0);
    }
    if let Some(r) = rows.get(shell.focus).filter(|r| !r.sub.is_empty()) {
        c.spawn((
            Node { width: cx.px(if controller { 760.0 } else { 520.0 }), padding: UiRect::all(cx.px(22.0)), border: UiRect::top(cx.px(3.0)), ..abs(None, Some(196.0), Some(64.0), None, cx) },
            BackgroundColor(cx.pal.panel),
            BorderColor::all(cx.pal.accent),
        ))
        .with_children(|p| {
            p.spawn((Text::new(r.sub.clone()), cx.regular(23.0), TextColor(TEXT)));
        });
    }
}

/// Barracks: his card large, then his record or his prestiges.
fn barracks_view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    header(c, cx, &[section(shell), "BARRACKS"]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
    })
    .with_children(|col| menu_rows(col, cx, rows, shell.focus, 520.0, false));
    let me = &cat.player;
    let icon = cx.pic(&[me.rank_picture.clone()]);
    c.spawn((
        Node {
            width: cx.px(1100.0),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(18.0),
            padding: UiRect::all(cx.px(28.0)),
            border: UiRect::top(cx.px(3.0)),
            ..abs(Some(680.0), Some(176.0), None, None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        let (title, mark) = (cards::mine(Kind::Card, &me.card), cards::mine(Kind::Emblem, &me.emblem));
        p.spawn(Node { column_gap: cx.px(24.0), align_items: AlignItems::Center, ..default() }).with_children(|top| {
            emblem(top, cx, &mark, 132.0);
            top.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(8.0), flex_grow: 1.0, ..default() }).with_children(|t| {
                banner(t, cx, &title, 560.0, 96.0, Some((me.name.as_str(), 48.0)));
                let prestige = if me.prestige > 0 { format!("PRESTIGE {}  ·  ", me.prestige) } else { String::new() };
                t.spawn(Node { column_gap: cx.px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|l| {
                    match icon {
                        Some(h) => {
                            l.spawn((ImageNode::new(h), Node { width: cx.px(34.0), height: cx.px(34.0), ..default() }));
                        }
                        None => {
                            l.spawn((Node { width: cx.px(34.0), height: cx.px(34.0), ..default() }, BackgroundColor(KEYCAP)));
                        }
                    }
                    l.spawn((Text::new(format!("{prestige}LEVEL {} OF {}", me.level, bo2_profile::progression::MAX_LEVEL)), cx.bold(24.0), TextColor(cx.pal.accent)));
                });
                t.spawn((Node { width: Val::Percent(100.0), height: cx.px(8.0), ..default() }, BackgroundColor(KEYCAP)))
                    .with_child((Node { width: Val::Percent(me.xp_frac * 100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(cx.pal.accent)));
                t.spawn((Text::new(format!("{} XP  ·  NEXT LEVEL AT {}", me.xp, me.xp_next)), cx.regular(19.0), TextColor(DIM)));
            });
        });
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(cx.pal.line)));
        let focused = rows.get(shell.focus).map(|r| r.act);
        if let Some(Act::Looks(kind)) = focused {
            look_strip(p, cx, kind, if kind == Kind::Card { &me.card } else { &me.emblem });
        } else if focused != Some(Act::Prestige) {
            p.spawn((Text::new("COMBAT RECORD"), cx.bold(22.0), TextColor(FAINT)));
            p.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: cx.px(16.0), row_gap: cx.px(16.0), ..default() }).with_children(|g| {
                for (k, v) in &me.record {
                    g.spawn((
                        Node { width: cx.px(240.0), flex_direction: FlexDirection::Column, padding: UiRect::all(cx.px(14.0)), ..default() },
                        BackgroundColor(cx.pal.panel_soft),
                    ))
                    .with_children(|b| {
                        b.spawn((Text::new(v.clone()), cx.bold(40.0), TextColor(WHITE)));
                        b.spawn((Text::new(k.clone()), cx.bold(17.0), TextColor(DIM)));
                    });
                }
            });
        } else {
            p.spawn((Text::new("PRESTIGE"), cx.bold(22.0), TextColor(FAINT)));
            let parts = [
                ("BLACK OPS", 1, bo2_profile::progression::BO1_PRESTIGES),
                ("MODERN WARFARE 2", 1 + bo2_profile::progression::BO1_PRESTIGES, bo2_profile::progression::MW2_PRESTIGES),
                ("MODERN WARFARE 3", 1 + bo2_profile::progression::BO1_PRESTIGES + bo2_profile::progression::MW2_PRESTIGES, bo2_profile::progression::MW3_PRESTIGES),
            ];
            for (title, first, count) in parts {
                p.spawn((Text::new(format!("{title}  ·  PRESTIGE {first}-{}", first + count - 1)), cx.bold(18.0), TextColor(DIM)));
                p.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: cx.px(6.0), row_gap: cx.px(6.0), ..default() }).with_children(|g| {
                    for n in first..first + count {
                        let pic = cx.pic(&[bo2_profile::progression::rank_picture(1, n)]);
                        let reached = me.prestige >= n;
                        g.spawn((
                            Node { width: cx.px(46.0), height: cx.px(46.0), border: UiRect::all(cx.px(if me.prestige == n { 2.0 } else { 0.0 })), ..default() },
                            BorderColor::all(cx.pal.accent),
                            BackgroundColor(cx.pal.panel_soft),
                        ))
                        .with_children(|b| {
                            if let Some(h) = pic {
                                b.spawn((
                                    ImageNode::new(h).with_color(Color::srgba(1.0, 1.0, 1.0, if reached { 1.0 } else { 0.28 })),
                                    Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                                ));
                            }
                        });
                    }
                });
            }
            let master = bo2_profile::progression::MAX_PRESTIGE;
            p.spawn((Text::new(format!("MASTER PRESTIGE  ·  PRESTIGE {master}")), cx.bold(18.0), TextColor(GOLD)));
        }
    });
}

/// Barracks: his calling card or emblem among its neighbours in the list
/// (ENTER opens the picker).
fn look_strip(p: &mut Kids, cx: &mut Ctx, kind: Kind, stat: &str) {
    let list = cards::list(kind);
    let now = cards::mine(kind, stat);
    let (at, n) = cards::place_in(&list, &now);
    let what = if kind == Kind::Card { "CALLING CARD" } else { "EMBLEM" };
    p.spawn(Node { justify_content: JustifyContent::SpaceBetween, ..default() }).with_children(|t| {
        t.spawn((Text::new(format!("{what}  ·  {}", cards::title(&now).to_uppercase())), cx.bold(22.0), TextColor(FAINT)));
        t.spawn((Text::new(format!("{at} OF {n}")), cx.bold(22.0), TextColor(DIM)));
    });
    let i = at.saturating_sub(1) as i32;
    let (w, h) = if kind == Kind::Card { (196.0, 44.0) } else { (96.0, 96.0) };
    let side = if kind == Kind::Card { 2 } else { 4 };
    p.spawn(Node { column_gap: cx.px(12.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
        for d in -side..=side {
            if list.is_empty() {
                break;
            }
            let name = &list[(i + d).rem_euclid(list.len() as i32) as usize];
            let on = d == 0;
            row.spawn((
                Node { padding: UiRect::all(cx.px(3.0)), border: UiRect::all(cx.px(2.0)), ..default() },
                BorderColor::all(if on { cx.pal.accent } else { Color::NONE }),
                BackgroundColor(if on { cx.pal.accent_a(0.18) } else { Color::NONE }),
            ))
            .with_children(|b| {
                let k = if on { 1.0 } else { 0.8 };
                if kind == Kind::Card {
                    banner(b, cx, name, w * k, h * k, None);
                } else {
                    emblem(b, cx, name, w * k);
                }
            });
        }
    });
    p.spawn((Text::new(format!("Press ENTER to choose your {} from every page.", what.to_lowercase())), cx.regular(19.0), TextColor(DIM)));
}


/// The CALL OF DUTY logo, drawn: the words in the black condensed weight,
/// a small OF stacked between them, filled with a brushed-silver sweep
/// (light at the top, a bright band under the middle, darker at the foot:
/// the words are drawn once per band, each band clipped to its strip) over
/// a soft dark shadow.
fn logo(c: &mut Kids, cx: &Ctx, left: f32, top: f32, px: f32) {
    // The logo's box: the words' line height, the caps sit in it.
    // (Wide enough for the whole of DUTY: the bands clip to it.)
    let (w, h) = (px * 4.8, px * 0.92);
    let words = |l: &mut Kids, color: Color, shadow: bool| {
        l.spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: cx.px(-px * 0.24),
            column_gap: cx.px(px * 0.035),
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|r| {
            let shade = |px: f32| TextShadow { offset: Vec2::new(0.0, px * 0.035 * cx.s), color: Color::srgba(0.0, 0.0, 0.0, if shadow { 0.85 } else { 0.0 }), ..default() };
            r.spawn((Text::new("CALL"), cx.black(px), TextColor(color), shade(px)));
            r.spawn((Text::new("OF"), cx.black(px * 0.40), TextColor(color), shade(px * 0.4), Node { margin: UiRect::top(cx.px(px * 0.17)), ..default() }));
            r.spawn((Text::new("DUTY"), cx.black(px), TextColor(color), shade(px)));
        });
    };
    c.spawn(Node { width: cx.px(w), height: cx.px(h), ..abs(Some(left), Some(top), None, None, cx) }).with_children(|l| {
        // The shadow under it all (the darkest copy, offset).
        words(l, Color::srgba(0.0, 0.0, 0.0, 0.0), true);
        // The silver: (where the band starts, 0..1 down the caps, its grey).
        const SWEEP: [(f32, f32); 6] = [(0.0, 0.97), (0.30, 0.86), (0.52, 0.70), (0.60, 0.92), (0.78, 0.66), (1.0, 0.50)];
        let bands = 22;
        for i in 0..bands {
            let t = (i as f32 + 0.5) / bands as f32;
            let k = SWEEP.windows(2).find(|p| t >= p[0].0 && t <= p[1].0).map_or(0.6, |p| {
                let f = (t - p[0].0) / (p[1].0 - p[0].0);
                p[0].1 + (p[1].1 - p[0].1) * f
            });
            let band = h / bands as f32;
            l.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: cx.px(band * i as f32),
                width: Val::Percent(100.0),
                height: cx.px(band + 0.6),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|b| {
                b.spawn(Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: cx.px(-band * i as f32), width: cx.px(w), height: cx.px(h), ..default() })
                    .with_children(|inner| words(inner, Color::srgb(k * 0.96, k * 0.975, k), false));
            });
        }
    });
}

/// The main menu, laid out as Infinite Warfare's: the logo at the top left,
/// thin rows under it each with a white bar at its left, the row in focus's
/// line under them, and EXIT at the foot; the menu photos behind.
fn main_menu(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row], online: &OnlineInfo) {
    let left = 150.0;
    // A deeper shade on the menus' side so the rows read over any photo.
    c.spawn((
        Node { width: cx.px(900.0), height: Val::Percent(100.0), ..abs(Some(0.0), Some(0.0), None, None, cx) },
        fade_right(Color::srgba(0.0, 0.01, 0.03, 0.55), Color::srgba(0.0, 0.01, 0.03, 0.0)),
    ));
    logo(c, cx, left, 150.0, 132.0);
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(5.0), ..abs(Some(left), Some(392.0), None, None, cx) }).with_children(|col| {
        for (i, r) in rows.iter().enumerate() {
            let on = i == shell.focus;
            col.spawn((
                Hit::Row(i),
                Button,
                Node { width: cx.px(492.0), height: cx.px(34.0), align_items: AlignItems::Center, column_gap: cx.px(14.0), ..default() },
                if on {
                    fade_right(Color::srgba(1.0, 1.0, 1.0, 0.20), Color::srgba(1.0, 1.0, 1.0, 0.04))
                } else {
                    fade_right(Color::srgba(0.0, 0.0, 0.0, 0.62), Color::srgba(0.0, 0.0, 0.0, 0.30))
                },
            ))
            .with_children(|row| {
                row.spawn((Node { width: cx.px(5.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(if on { WHITE } else { Color::srgba(1.0, 1.0, 1.0, 0.82) })));
                row.spawn((Text::new(r.label.clone()), cx.regular(25.0), TextColor(if on { WHITE } else { TEXT.with_alpha(0.80) })));
            });
        }
        if let Some(r) = rows.get(shell.focus) {
            col.spawn((Text::new(r.sub.clone()), cx.regular(25.0), TextColor(WHITE), Node { margin: UiRect::top(cx.px(14.0)), ..default() }));
        }
    });
    // EXIT at the foot (a tab with the same white bar), the server's state
    // at the right.
    c.spawn((
        Hit::Back,
        Button,
        Node { width: cx.px(236.0), height: cx.px(38.0), align_items: AlignItems::Center, column_gap: cx.px(14.0), ..abs(Some(left - 16.0), None, None, Some(18.0), cx) },
        fade_right(Color::srgba(0.0, 0.0, 0.0, 0.70), Color::srgba(0.0, 0.0, 0.0, 0.40)),
    ))
    .with_children(|b| {
        b.spawn((Node { width: cx.px(5.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(WHITE)));
        b.spawn((Node { flex_grow: 1.0, justify_content: JustifyContent::Center, ..default() },)).with_child((Text::new("EXIT"), cx.regular(24.0), TextColor(WHITE)));
    });
    let (dot, label) = match online.status {
        Status::Online => (GREEN, format!("ONLINE  ·  {} PLAYERS", fmt_count(online.players))),
        Status::Connecting => (GOLD, "CONNECTING".to_owned()),
        Status::Unreachable => (ENEMY, "SERVER UNREACHABLE".to_owned()),
        Status::Offline => (FAINT, "OFFLINE  ·  LOCAL PLAY".to_owned()),
    };
    c.spawn(Node { column_gap: cx.px(24.0), align_items: AlignItems::Center, ..abs(None, None, Some(72.0), Some(22.0), cx) }).with_children(|r| {
        if !cx.pad {
            hint(r, cx, "F10", "", "GAME FOLDERS");
        }
        r.spawn(Node { column_gap: cx.px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|s| {
            s.spawn((Node { width: cx.px(9.0), height: cx.px(9.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() }, BackgroundColor(dot)));
            s.spawn((Text::new(label), cx.bold(18.0), TextColor(DIM)));
        });
    });
}

/// Multiplayer, as Infinite Warfare's MISSION SELECT: the title, the rows
/// at the left with the line about the one in focus, a featured map framed
/// at the bottom left, his card and his party at the right.
fn multiplayer(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    header(c, cx, &["MULTIPLAYER", "MATCH SELECT"]);
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(6.0), ..abs(Some(126.0), Some(196.0), None, None, cx) })
        .with_children(|col| menu_rows(col, cx, rows, shell.focus, 470.0, false));
    featured(c, cx, cat);
    party(c, cx, cat);
}

/// A map from the rotation, framed at the bottom left (the slot Infinite
/// Warfare keeps for its Message of the Day), changing every few seconds.
pub(crate) fn featured(c: &mut Kids, cx: &mut Ctx, cat: &Catalog) {
    if cat.maps.is_empty() {
        return;
    }
    let at = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) / 600) as usize % cat.maps.len();
    let m = &cat.maps[at];
    let (w, h) = (326.0, 196.0);
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(6.0), ..abs(Some(132.0), None, None, Some(96.0), cx) }).with_children(|col| {
        col.spawn((Node { border: UiRect::all(cx.px(3.0)), ..default() }, BorderColor::all(WHITE))).with_children(|f| {
            picture_box(f, cx, &m.pictures, w, h, ());
        });
        col.spawn(Node { column_gap: cx.px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|t| {
            t.spawn((Text::new("FEATURED MAP"), cx.regular(20.0), TextColor(DIM)));
            t.spawn((Text::new(m.title.clone()), cx.bold(22.0), TextColor(WHITE)));
            t.spawn((Text::new(m.origin.to_uppercase()), cx.regular(18.0), TextColor(cx.pal.accent)));
        });
    });
}

/// His party under his card: a line each (an arrow at his own), rank and
/// level, then the name.
fn party(c: &mut Kids, cx: &mut Ctx, cat: &Catalog) {
    let me = &cat.player;
    let icon = cx.pic(&[me.rank_picture.clone()]);
    c.spawn(Node { width: cx.px(640.0), flex_direction: FlexDirection::Column, ..abs(None, Some(196.0), Some(90.0), None, cx) }).with_children(|list| {
        list.spawn((
            Node { height: cx.px(38.0), column_gap: cx.px(14.0), align_items: AlignItems::Center, border: UiRect::bottom(cx.px(1.0)), ..default() },
            BorderColor::all(WHITE.with_alpha(0.35)),
        ))
        .with_children(|r| {
            pointer(r, cx);
            if let Some(h) = icon {
                r.spawn((ImageNode::new(h), Node { width: cx.px(28.0), height: cx.px(28.0), ..default() }));
            }
            r.spawn((Node { width: cx.px(44.0), ..default() },)).with_child((Text::new(me.level.to_string()), cx.regular(24.0), TextColor(WHITE)));
            r.spawn((Text::new(me.name.clone()), cx.regular(24.0), TextColor(WHITE)));
        });
    });
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
    let book = super::classes::book().map_or(0, |b| std::sync::Arc::as_ptr(&b) as usize);
    let key = format!(
        "{:?}|{:p}|{}|{online:?}|{area:?}|{pad}|{starting:?}|{tick}|{book}|{}|{:?}|{}|{:?}",
        *shell,
        std::sync::Arc::as_ptr(&cat),
        pics.count(),
        super::options::snapshot().revision,
        super::zombies::state(),
        super::online::generation(),
        crate::online_match::room()
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
        w: area.x / s,
        h: area.y / s,
        fonts: &fonts,
        pics: &mut pics,
        icons: icons.as_deref(),
        images: &mut images,
        pad,
        pal: palette(&shell),
        still: false,
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
                Screen::Main => main_menu(c, &mut cx, &shell, &rows, &online),
                Screen::Multiplayer => multiplayer(c, &mut cx, &shell, &cat, &rows),
                Screen::Lobby => lobby(c, &mut cx, &shell, &cat, &rows),
                Screen::MapPick => map_picker(c, &mut cx, &shell, &cat),
                Screen::ModePick => mode_picker(c, &mut cx, &shell, &cat, &rows),
                Screen::Options => options_view(c, &mut cx, &shell, &rows),
                Screen::Barracks => barracks_view(c, &mut cx, &shell, &cat, &rows),
                Screen::Friends => super::friends::view(c, &mut cx, &shell, &rows),
                Screen::Lobbies if shell.searching => super::matchmaking::search_view(c, &mut cx, &shell, &cat, &rows),
                Screen::Lobbies => super::lobbies::view(c, &mut cx, &shell, &rows),
                Screen::MatchFilter => super::matchmaking::view(c, &mut cx, &shell, &cat, &rows),
                Screen::Room => super::room::view(c, &mut cx, &shell, &cat, &rows),
                Screen::CardPick | Screen::EmblemPick => super::looks::view(c, &mut cx, &shell, &cat, &rows),
                Screen::Classes | Screen::ClassEdit | Screen::ItemPick => {
                    if let Some(book) = super::classes::book() {
                        super::cac::view(c, &mut cx, &shell, &book, &rows);
                    }
                }
                Screen::Zombies | Screen::ZmMapPick | Screen::ZmStartPick | Screen::ZmLobby | Screen::ZmRecord => {
                    super::zombies_view::view(c, &mut cx, &shell, &rows);
                }
                Screen::WzMenu | Screen::WzLobby | Screen::WzModePick => super::wz_view::view(c, &mut cx, &shell, &cat, &rows),
            }
            // (The main menu has its own foot, and no player card; the
            // calling card and emblem pickers show his card in their middle.)
            if shell.screen.is_zombies() {
                super::zombies_view::zm_card(c, &mut cx, &cat.player);
            } else if !matches!(shell.screen, Screen::Main | Screen::CardPick | Screen::EmblemPick) {
                player_card(c, &mut cx, &cat.player);
            }
            if shell.screen != Screen::Main {
                footer(c, &mut cx, &shell, &cat, &online);
            }
            if let Some(ask) = shell.ask {
                question(c, &mut cx, ask, shell.ask_focus);
            }
            if let Some((note, _)) = &shell.note {
                c.spawn((
                    Node { padding: UiRect::axes(cx.px(22.0), cx.px(10.0)), border: UiRect::left(cx.px(3.0)), ..abs(Some(72.0), None, None, Some(84.0), &cx) },
                    BackgroundColor(cx.pal.panel),
                    BorderColor::all(GOLD),
                ))
                .with_child((Text::new(note.clone()), cx.bold(21.0), TextColor(WHITE)));
            }
        });
}

/// A yes / no question over the screen: the screen dimmed, a box in the
/// middle with the question, what it means, and YES / NO (or OK alone for
/// a notice).
pub(crate) fn question(c: &mut Kids, cx: &mut Ctx, ask: super::Ask, focus: usize) {
    let (title, body) = ask.text();
    let [r, g, b] = cx.pal.shade;
    c.spawn((
        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        BackgroundColor(Color::srgba(r, g, b, 0.72)),
        GlobalZIndex(40),
    ));
    let (w, h) = (720.0, 330.0);
    c.spawn((
        Node {
            width: cx.px(w),
            min_height: cx.px(h),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(cx.px(32.0)),
            row_gap: cx.px(10.0),
            border: UiRect::all(cx.px(1.0)).with_top(cx.px(4.0)),
            ..abs(Some((cx.w - w) * 0.5), Some((cx.h - h) * 0.5), None, None, cx)
        },
        BackgroundColor(cx.pal.panel.with_alpha(0.97)),
        BorderColor { top: cx.pal.accent, ..BorderColor::all(cx.pal.line) },
        GlobalZIndex(41),
    ))
    .with_children(|p| {
        p.spawn((Text::new(title), cx.bold(56.0), TextColor(WHITE)));
        p.spawn((Text::new(body), cx.regular(23.0), TextColor(TEXT)));
        p.spawn(Node { flex_grow: 1.0, min_height: cx.px(24.0), ..default() });
        for (i, &label) in ask.buttons().iter().enumerate() {
            let on = i == focus;
            let mut e = p.spawn((
                Hit::Ask(i),
                Button,
                Node {
                    width: cx.px(300.0),
                    height: cx.px(52.0),
                    padding: UiRect::horizontal(cx.px(20.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::left(cx.px(4.0)),
                    ..default()
                },
                BorderColor::all(if on { cx.pal.accent } else { Color::NONE }),
            ));
            if on {
                e.insert(fade_right(cx.pal.accent_a(0.30), cx.pal.accent_a(0.0)));
            } else {
                e.insert(BackgroundColor(cx.pal.panel_soft));
            }
            e.with_child((Text::new(label), cx.bold(28.0), TextColor(if on { WHITE } else { TEXT.with_alpha(0.72) })));
        }
    });
}
