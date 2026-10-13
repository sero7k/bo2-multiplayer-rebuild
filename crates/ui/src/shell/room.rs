//! bo2mp shell: the waiting room. After he joins a lobby on the server
//! (FRIENDS or FIND A LOBBY) the shell shows its map, mode and players until
//! its host starts the match; Esc leaves it.

use super::bridge::Catalog;
use super::view::{Ctx, Kids};
use super::{Row, Shell};
use crate::online_match::RoomView;

/// A map's name from its zone name (`t6:mp_nuketown_2020` -> `NUKETOWN 2020`).
fn map_title(map: &str) -> String {
    map.split(':').next_back().unwrap_or(map).trim_start_matches("mp_").replace('_', " ").to_uppercase()
}

pub(crate) fn rows(shell: &Shell, c: &Catalog) -> Vec<Row> {
    super::matchmaking::lobby_rows(shell, c, false)
}

/// The waiting room, as FIND A MATCH's matchmaking lobby: the mode, how many
/// places are left until the host starts, the players, the map.
pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, cat: &Catalog, rows: &[Row]) {
    let room = crate::online_match::room().unwrap_or_else(RoomView::default);
    let mode = cat.mode(&room.mode).map_or_else(|| room.mode.to_uppercase(), |m| m.title.clone());
    let stem = room.map.split(':').next_back().unwrap_or(&room.map);
    let map = cat.map(stem).or_else(|| cat.maps.iter().find(|m| m.ui_name.starts_with(stem)));
    let pictures = map.map(|m| m.pictures.clone()).unwrap_or_default();
    let map_name = map.map_or_else(|| map_title(&room.map), |m| m.title.to_uppercase());
    let max = usize::from(room.max_players).max(room.players.len()).clamp(2, 18);
    let left = max.saturating_sub(room.players.len());
    let status = if left == 0 {
        "LOBBY FULL: WAITING FOR THE HOST".to_owned()
    } else {
        format!("WAITING FOR {left} MORE PLAYER{}", if left == 1 { "" } else { "S" })
    };
    let places: Vec<super::matchmaking::Place> = room
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| super::matchmaking::Place {
            level: p.eq_ignore_ascii_case(&cat.player.name).then_some(cat.player.level),
            name: p.clone(),
            me: p.eq_ignore_ascii_case(&cat.player.name),
            host: i == 0,
        })
        .collect();
    let sub = format!("{} · the host starts the match. Esc leaves.", room.name);
    super::matchmaking::lobby_view(c, cx, shell, rows, &mode, (&status, &sub), (&pictures, &map_name, &mode), &places, max);
}

#[cfg(test)]
mod tests {
    #[test]
    fn map_titles() {
        assert_eq!(super::map_title("iw3:mp_crash"), "CRASH");
    }
}
