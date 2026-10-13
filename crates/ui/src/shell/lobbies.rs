//! bo2mp shell: FIND A LOBBY. Every lobby on the server (the master's list,
//! refreshed every 15 seconds): open ones first, then full, closed or
//! playing ones. Enter on an open lobby joins it (`online_match::queue_join`);
//! he loads in when its host starts the match. Lobbies on maps he does not
//! have installed say which game they need.

use std::sync::Mutex;

use bevy::prelude::*;
use net::MasterAdvert;
use net::master_protocol::AdvertId;

use super::theme::*;
use super::view::{Ctx, Kids, abs, header, menu_rows};
use super::{Act, Row, Shell};

/// The shell preview's stand-in list.
static PREVIEW: Mutex<Option<Vec<MasterAdvert>>> = Mutex::new(None);

pub(crate) fn preview(list: Vec<MasterAdvert>) {
    if let Ok(mut p) = PREVIEW.lock() {
        *p = Some(list);
    }
}

/// Every lobby on the server (or the preview's), unsorted by any search.
pub(crate) fn all() -> Vec<MasterAdvert> {
    list()
}

/// The shell preview is showing a stand-in list.
pub(crate) fn previewing() -> bool {
    PREVIEW.lock().is_ok_and(|p| p.is_some())
}

fn list() -> Vec<MasterAdvert> {
    let mut list = PREVIEW.lock().ok().and_then(|p| p.clone()).unwrap_or_else(super::online::adverts);
    list.sort_by_key(|a| (!joinable(a), a.missing.0 != 0, std::cmp::Reverse(a.players), a.name.to_lowercase()));
    list
}

/// He can join it now.
pub(crate) fn joinable(a: &MasterAdvert) -> bool {
    !a.in_match && !a.locked && a.players < a.max_players && a.missing.0 == 0
}

/// A map's name from its zone name (`t6:mp_nuketown_2020` -> `NUKETOWN 2020`).
fn map_title(map: &str) -> String {
    map.split(':').next_back().unwrap_or(map).trim_start_matches("mp_").replace('_', " ").to_uppercase()
}

/// The games a lobby needs that he does not have, by name.
fn games(missing: net::master_protocol::ContentFlags) -> String {
    net::content_names(missing)
        .split(',')
        .map(|g| match g {
            "iw3" => "CALL OF DUTY 4",
            "t4" => "WORLD AT WAR",
            "iw4" => "MODERN WARFARE 2",
            "iw5" => "MODERN WARFARE 3",
            "t5" => "BLACK OPS",
            "t6" => "BLACK OPS II",
            _ => "ANOTHER GAME",
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn status(a: &MasterAdvert) -> String {
    if a.missing.0 != 0 {
        format!("NEEDS {}", games(a.missing))
    } else if a.in_match {
        "IN A MATCH".to_owned()
    } else if a.locked {
        "CLOSED".to_owned()
    } else if a.players >= a.max_players {
        "FULL".to_owned()
    } else {
        format!("{}/{}", a.players, a.max_players)
    }
}

/// The lobbies shown: every one, or FIND A MATCH's search (open lobbies of
/// the modes ticked).
fn shown(shell: &Shell) -> Vec<MasterAdvert> {
    let list = list();
    if !shell.searching {
        return list;
    }
    list.into_iter().filter(|a| joinable(a) && super::matchmaking::passes(shell, &a.map, &a.mode)).collect()
}

pub(crate) fn rows(shell: &Shell) -> Vec<Row> {
    if shell.searching {
        return super::matchmaking::lobby_rows(true);
    }
    let list = shown(shell);
    if list.is_empty() && shell.searching {
        let online = crate::online_match::online_available() || PREVIEW.lock().is_ok_and(|p| p.is_some());
        return vec![
            Row {
                label: "NO MATCHES FOUND".to_owned(),
                sub: if online {
                    "Nobody is in an open lobby of these modes yet. Searching again every 15 seconds.".to_owned()
                } else {
                    "No server set: fill in online.cfg next to run-bo2.cmd (see docs\\ONLINE.md).".to_owned()
                },
                value: None,
                act: Act::Info,
            },
            Row {
                label: "HOST A MATCH".to_owned(),
                sub: "Open your own lobby in this playlist: the next player searching finds it.".to_owned(),
                value: None,
                act: Act::HostMatch,
            },
        ];
    }
    if list.is_empty() {
        let online = crate::online_match::online_available() || PREVIEW.lock().is_ok_and(|p| p.is_some());
        return vec![Row {
            label: "NO LOBBIES".to_owned(),
            sub: if online {
                "Nobody is in a lobby on the server. Start a custom game and your friends can join it.".to_owned()
            } else {
                "No server set: fill in online.cfg next to run-bo2.cmd (see docs\\ONLINE.md).".to_owned()
            },
            value: None,
            act: Act::Info,
        }];
    }
    list.iter()
        .take(10)
        .map(|a| Row {
            label: a.name.to_uppercase(),
            sub: format!("{}  ·  {}", map_title(&a.map), a.mode.to_uppercase()),
            value: Some(status(a)),
            act: Act::Lobby(a.id),
        })
        .collect()
}

/// What Enter on a lobby does: join it when he can, and says so.
pub(crate) fn lobby_note(id: AdvertId) -> String {
    let Some(a) = list().into_iter().find(|a| a.id == id) else {
        return "That lobby has closed.".to_owned();
    };
    if a.missing.0 != 0 {
        return format!("You need {} installed to play {}.", games(a.missing), map_title(&a.map));
    }
    if !joinable(&a) {
        return format!("{} is {}.", a.name, status(&a).to_lowercase());
    }
    crate::online_match::queue_join_room(a.id, a.map.clone(), a.mode.clone());
    format!("Joining {} on {}. You load in when the host starts the match.", a.name, map_title(&a.map))
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    if shell.searching {
        header(c, cx, &["FIND A MATCH", "SEARCH RESULTS"]);
    } else {
        header(c, cx, &["MULTIPLAYER", "FIND A LOBBY"]);
    }
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(8.0), ..abs(Some(72.0), Some(176.0), None, None, cx) })
        .with_children(|col| menu_rows(col, cx, rows, shell.focus, 720.0, false));
    let list = shown(shell);
    let open = list.iter().filter(|a| joinable(a)).count();
    let players: u32 = list.iter().map(|a| u32::from(a.players)).sum();
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
        p.spawn((Text::new("LOBBIES ON THE SERVER"), cx.bold(22.0), TextColor(FAINT)));
        p.spawn((Text::new(format!("{open} OPEN  ·  {} IN ALL  ·  {players} PLAYERS", list.len())), cx.bold(30.0), TextColor(WHITE)));
        if let Some(Act::Lobby(id)) = rows.get(shell.focus).map(|r| r.act)
            && let Some(a) = list.iter().find(|a| a.id == id)
        {
            p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), margin: UiRect::vertical(cx.px(8.0)), ..default() }, BackgroundColor(cx.pal.line)));
            p.spawn((Text::new(a.name.to_uppercase()), cx.bold(30.0), TextColor(WHITE)));
            p.spawn((Text::new(format!("{}  ·  {}", map_title(&a.map), a.mode.to_uppercase())), cx.bold(24.0), TextColor(cx.pal.accent)));
            p.spawn((Text::new(format!("{}/{} PLAYERS{}", a.players, a.max_players, if joinable(a) { "  ·  OPEN".to_owned() } else { format!("  ·  {}", status(a)) })), cx.bold(21.0), TextColor(DIM)));
            p.spawn((
                Text::new(if joinable(a) { "Press Enter to join. You load in when the host starts the match." } else { "You can't join this lobby right now." }),
                cx.regular(20.0),
                TextColor(DIM),
            ));
        } else {
            p.spawn((Text::new("Lobbies refresh every 15 seconds."), cx.regular(20.0), TextColor(DIM)));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advert(n: u8, players: u8, in_match: bool, missing: u8) -> MasterAdvert {
        MasterAdvert {
            id: AdvertId([n; 16]),
            name: format!("lobby {n}"),
            map: "t6:mp_raid".to_owned(),
            mode: "tdm".to_owned(),
            players,
            max_players: 12,
            locked: false,
            in_match,
            requires: Default::default(),
            missing: master_protocol_flags(missing),
        }
    }

    fn master_protocol_flags(f: u8) -> net::master_protocol::ContentFlags {
        net::master_protocol::ContentFlags(f)
    }

    #[test]
    fn statuses_and_titles() {
        assert_eq!(status(&advert(1, 3, false, 0)), "3/12");
        assert_eq!(status(&advert(1, 12, false, 0)), "FULL");
        assert_eq!(status(&advert(1, 3, true, 0)), "IN A MATCH");
        assert_eq!(status(&advert(1, 3, false, net::CONTENT_T4)), "NEEDS WORLD AT WAR");
        assert_eq!(map_title("t6:mp_nuketown_2020"), "NUKETOWN 2020");
        assert!(joinable(&advert(1, 3, false, 0)));
        assert!(!joinable(&advert(1, 3, false, net::CONTENT_IW3)));
    }
}
