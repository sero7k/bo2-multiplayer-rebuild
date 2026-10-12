//! bo2mp shell: Barracks' calling card and emblem pickers, laid out as
//! Modern Warfare 2's EMBLEM and TITLE screens but in the shell's look: the
//! card he would have at the top in the middle (the calling card, his name
//! on it, the emblem at its right), PAGE n/m at the right, a page of them
//! under it (emblems 8 by 6, calling cards in 3 columns of 9 with their
//! names on them) and a big arrow each side to turn the page. ENTER keeps
//! the one in focus (his stats), ESC goes back without a change.

use bevy::prelude::*;

use super::bridge::Catalog;
use super::cards::{self, Kind};
use super::theme::*;
use super::view::{Ctx, Kids, abs, banner, emblem, header};
use super::{Hit, Row, Shell, look_kind};

/// The grids' tiles (design units at 1080 lines) and where they start.
const EMBLEM_TILE: f32 = 96.0;
const EMBLEM_GAP: (f32, f32) = (18.0, 14.0);
const CARD_TILE: (f32, f32) = (436.0, 58.0);
const CARD_GAP: (f32, f32) = (30.0, 13.0);
const GRID_TOP: f32 = 318.0;

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    let kind = look_kind(shell.screen);
    let list = cards::list(kind);
    let pages = cards::pages(kind, list.len());
    let page = shell.page.min(pages - 1);
    let items = cards::page_items(kind, &list, page);
    let me = &cat.player;
    let (my_card, my_emblem) = (cards::mine(Kind::Card, &me.card), cards::mine(Kind::Emblem, &me.emblem));
    let equipped = if kind == Kind::Card { &my_card } else { &my_emblem };
    let what = if kind == Kind::Card { "CALLING CARD" } else { "EMBLEM" };
    header(c, cx, &["BARRACKS", what]);
    // PAGE n/m at the right.
    c.spawn((Node { align_items: AlignItems::Baseline, column_gap: cx.px(10.0), ..abs(None, Some(64.0), Some(72.0), None, cx) },)).with_children(|t| {
        t.spawn((Text::new("PAGE"), cx.bold(26.0), TextColor(DIM)));
        t.spawn((Text::new(format!("{}/{}", page + 1, pages)), cx.bold(40.0), TextColor(WHITE)));
    });
    // The card he would have: the one in focus.
    let focused = items.get(shell.focus).cloned().unwrap_or_else(|| equipped.clone());
    let (card, mark) = if kind == Kind::Card { (focused.clone(), my_emblem.clone()) } else { (my_card.clone(), focused.clone()) };
    c.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: cx.px(8.0),
        ..abs(Some(0.0), Some(118.0), None, None, cx)
    })
    .with_children(|col| {
        preview(col, cx, &card, &mark, &me.name);
        let (at, n) = cards::place_in(&list, &focused);
        let game = cards::label(&focused).split(" · ").next().unwrap_or("").to_owned();
        let state = if focused == *equipped { "  ·  EQUIPPED" } else { "" };
        col.spawn((Text::new(format!("{}  ·  {game}  ·  {at} OF {n}{state}", cards::title(&focused).to_uppercase())), cx.bold(21.0), TextColor(if state.is_empty() { DIM } else { cx.pal.accent })));
    });
    // The page.
    let (cols, tw, th, gx, gy) = match kind {
        Kind::Emblem => (cards::columns(kind), EMBLEM_TILE, EMBLEM_TILE, EMBLEM_GAP.0, EMBLEM_GAP.1),
        Kind::Card => (cards::columns(kind), CARD_TILE.0, CARD_TILE.1, CARD_GAP.0, CARD_GAP.1),
    };
    let grid_w = cols as f32 * tw + (cols as f32 - 1.0) * gx;
    let grid_rows = cards::per_page(kind) / cols;
    let grid_h = grid_rows as f32 * th + (grid_rows as f32 - 1.0) * gy;
    let grid_left = ((cx.w - grid_w) * 0.5).round();
    c.spawn(Node { width: cx.px(grid_w + 1.0), flex_wrap: FlexWrap::Wrap, column_gap: cx.px(gx), row_gap: cx.px(gy), ..abs(Some(grid_left), Some(GRID_TOP), None, None, cx) })
        .with_children(|g| {
            for (k, name) in items.iter().enumerate() {
                let on = k == shell.focus;
                let mine = name == equipped;
                let title = rows.get(k).map_or_else(|| cards::title(name), |r| r.label.clone());
                tile(g, cx, kind, name, &title, tw, th, k, on, mine);
            }
        });
    // The page arrows, each side of the grid.
    let mid = GRID_TOP + grid_h * 0.5;
    let more = pages > 1;
    for (by, glyph, left) in [(-1, "‹", grid_left - 190.0), (1, "›", grid_left + grid_w + 70.0)] {
        c.spawn((
            Hit::PageStep(by),
            Button,
            Node { width: cx.px(120.0), height: cx.px(220.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..abs(Some(left), Some(mid - 120.0), None, None, cx) },
        ))
        .with_child((Text::new(glyph), cx.light(230.0), TextColor(if more { WHITE.with_alpha(0.9) } else { FAINT }), TextShadow { offset: Vec2::splat(3.0 * cx.s), color: Color::srgba(0.0, 0.0, 0.0, 0.6), ..default() }));
    }
}

/// The card as it would show: his calling card with its name at the top and
/// his name at the foot, his emblem at the right, in the shell's frame.
fn preview(c: &mut Kids, cx: &mut Ctx, card: &str, mark: &str, name: &str) {
    let (w, h) = (560.0, 120.0);
    c.spawn((
        Node { padding: UiRect::all(cx.px(6.0)), border: UiRect::all(cx.px(1.0)), column_gap: cx.px(6.0), ..default() },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent_a(0.65)),
    ))
    .with_children(|f| {
        f.spawn(Node { width: cx.px(w), height: cx.px(h), ..default() }).with_children(|b| {
            banner(b, cx, card, w, h, None);
            b.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::axes(cx.px(16.0), cx.px(8.0)),
                    ..default()
                },
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.0), Val::Percent(45.0)),
                    ColorStop::new(Color::srgba(0.0, 0.0, 0.0, 0.62), Val::Percent(100.0)),
                ])),
            ))
            .with_children(|t| {
                t.spawn((Text::new(cards::title(card)), cx.bold(28.0), TextColor(WHITE), shadow(cx)));
                t.spawn((Text::new(name.to_owned()), cx.bold(38.0), TextColor(WHITE), shadow(cx)));
            });
        });
        emblem(f, cx, mark, h);
    });
}

fn shadow(cx: &Ctx) -> TextShadow {
    TextShadow { offset: Vec2::splat(2.0 * cx.s), color: Color::srgba(0.0, 0.0, 0.0, 0.85), ..default() }
}

/// One of the page: an emblem in a square, or a calling card with its name
/// across it. The one in focus is framed in the accent and lit; the one he
/// has is marked.
#[allow(clippy::too_many_arguments)]
fn tile(g: &mut Kids, cx: &mut Ctx, kind: Kind, name: &str, title: &str, w: f32, h: f32, k: usize, on: bool, mine: bool) {
    let edge = if on { 3.0 } else { 1.0 };
    g.spawn((
        Hit::Row(k),
        Button,
        Node { width: cx.px(w), height: cx.px(h), border: UiRect::all(cx.px(edge)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BorderColor::all(if on { cx.pal.accent } else if mine { cx.pal.accent_a(0.55) } else { cx.pal.line }),
        BackgroundColor(if on { cx.pal.accent_a(0.22) } else { cx.pal.panel_soft }),
    ))
    .with_children(|t| {
        let (iw, ih) = (w - edge * 2.0, h - edge * 2.0);
        match kind {
            Kind::Emblem => emblem(t, cx, name, ih.min(iw) - 8.0),
            Kind::Card => {
                t.spawn(Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), ..default() }).with_children(|b| {
                    banner(b, cx, name, iw, ih, None);
                });
                // (Unlit when not in focus, so the names read.)
                if !on {
                    t.spawn((Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.30))));
                }
                t.spawn((Text::new(title.to_owned()), cx.bold(25.0), TextColor(if on { WHITE } else { TEXT }), shadow(cx), TextLayout::linebreak(bevy::text::LineBreak::NoWrap)));
            }
        }
        if mine {
            t.spawn((
                Node { position_type: PositionType::Absolute, right: cx.px(4.0), top: cx.px(4.0), width: cx.px(12.0), height: cx.px(12.0), border_radius: BorderRadius::all(Val::Percent(50.0)), ..default() },
                BackgroundColor(cx.pal.accent),
            ));
        }
    });
}
