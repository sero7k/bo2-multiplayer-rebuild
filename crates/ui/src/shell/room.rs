//! bo2mp shell: the waiting room. After he joins a lobby on the server
//! (FRIENDS or FIND A LOBBY) the shell shows its map, mode and players until
//! its host starts the match; Esc leaves it.

use bevy::prelude::*;

use super::theme::*;
use super::view::{Ctx, Kids, abs, header, menu_rows};
use super::{Act, Row, Shell};
use crate::online_match::RoomView;

/// A map's name from its zone name (`t6:mp_nuketown_2020` -> `NUKETOWN 2020`).
fn map_title(map: &str) -> String {
    map.split(':').next_back().unwrap_or(map).trim_start_matches("mp_").replace('_', " ").to_uppercase()
}

pub(crate) fn rows(_shell: &Shell) -> Vec<Row> {
    let Some(room) = crate::online_match::room() else { return Vec::new() };
    room.players
        .iter()
        .enumerate()
        .map(|(i, p)| Row {
            label: p.clone(),
            sub: String::new(),
            value: (i == 0).then(|| "HOST".to_owned()),
            act: Act::Info,
        })
        .collect()
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    let room = crate::online_match::room().unwrap_or_else(RoomView::default);
    header(c, cx, &["ONLINE", "LOBBY"]);
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(8.0), ..abs(Some(72.0), Some(176.0), None, None, cx) })
        .with_children(|col| menu_rows(col, cx, rows, shell.focus, 720.0, false));
    c.spawn((
        Node {
            width: cx.px(860.0),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(14.0),
            padding: UiRect::all(cx.px(28.0)),
            border: UiRect::top(cx.px(3.0)),
            ..abs(Some(880.0), Some(176.0), None, None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        p.spawn((Text::new(room.name.to_uppercase()), cx.bold(22.0), TextColor(FAINT)));
        p.spawn((Text::new(map_title(&room.map)), cx.bold(72.0), TextColor(WHITE)));
        p.spawn((Text::new(room.mode.to_uppercase()), cx.bold(30.0), TextColor(cx.pal.accent)));
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), margin: UiRect::vertical(cx.px(8.0)), ..default() }, BackgroundColor(cx.pal.line)));
        p.spawn((Text::new(format!("{}/{} PLAYERS", room.players.len(), room.max_players)), cx.bold(24.0), TextColor(WHITE)));
        p.spawn((Text::new("WAITING FOR THE HOST TO START THE MATCH"), cx.bold(24.0), TextColor(DIM)));
        p.spawn((Text::new("Esc leaves the lobby."), cx.regular(20.0), TextColor(DIM)));
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn map_titles() {
        assert_eq!(super::map_title("iw3:mp_crash"), "CRASH");
    }
}
