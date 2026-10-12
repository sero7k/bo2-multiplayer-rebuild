//! bo2mp shell: FRIENDS. His own friend code (Options > ACCOUNT shows it
//! too), ADD FRIEND (type an 8-digit code, Enter adds it), and his friends
//! as the master last saw them: ONLINE, IN A LOBBY (its map, mode and
//! players), IN A MATCH, or OFFLINE. Del removes the friend in focus.
//!
//! The master answers where his friends are on each lobby-list refresh
//! (`net::MasterBrowser`, every 15 seconds while the front end is up); with
//! no master server set, everyone shows as offline.

use std::sync::Mutex;

use bevy::prelude::*;
use frame::GameSettings;
use net::FriendStatus;

use super::theme::*;
use super::view::{Ctx, Kids, abs, header, menu_rows};
use super::{Act, Row, Shell};

/// His friends as the master last saw them (`online::poll_online` keeps it).
static SEEN: Mutex<Vec<FriendStatus>> = Mutex::new(Vec::new());

/// Friend codes to add (true) or remove (false), applied by
/// `options::sync_settings` (the one system that writes the settings).
static EDITS: Mutex<Vec<(bool, u32)>> = Mutex::new(Vec::new());

/// The shell preview's stand-ins: his code, his friends and what the
/// master says of them.
static PREVIEW: Mutex<Option<(u32, Vec<u32>, Vec<FriendStatus>)>> = Mutex::new(None);

pub(crate) fn set_seen(friends: Vec<FriendStatus>) {
    if let Ok(mut s) = SEEN.lock() {
        *s = friends;
    }
}

pub(crate) fn queue_add(code: u32) {
    if let Ok(mut e) = EDITS.lock() {
        e.push((true, code));
    }
}

pub(crate) fn queue_remove(code: u32) {
    if let Ok(mut e) = EDITS.lock() {
        e.push((false, code));
    }
}

/// The queued edits on his settings (true when any changed them).
pub(crate) fn apply_edits(s: &mut GameSettings) -> bool {
    let edits = EDITS.lock().map(|mut e| std::mem::take(&mut *e)).unwrap_or_default();
    let mut changed = false;
    for (add, code) in edits {
        if add {
            if GameSettings::is_friend_code(code) && code != s.friend_code && !s.friends.contains(&code) {
                s.friends.push(code);
                changed = true;
            }
        } else {
            let before = s.friends.len();
            s.friends.retain(|&c| c != code);
            changed |= s.friends.len() != before;
        }
    }
    changed
}

pub(crate) fn preview(mine: u32, friends: Vec<u32>, seen: Vec<FriendStatus>) {
    if let Ok(mut p) = PREVIEW.lock() {
        *p = Some((mine, friends, seen));
    }
}

/// His code, his friends' codes, and what the master says of each.
fn state() -> (u32, Vec<(u32, Option<FriendStatus>)>) {
    let (mine, codes, seen) = match PREVIEW.lock().ok().and_then(|p| p.clone()) {
        Some(p) => p,
        None => {
            let s = super::options::snapshot();
            (s.friend_code, s.friends, SEEN.lock().map(|s| s.clone()).unwrap_or_default())
        }
    };
    let list = codes.into_iter().map(|c| (c, seen.iter().find(|f| f.code == c).cloned())).collect();
    (mine, list)
}

/// A map's name from its zone name (`mp_nuketown_2020` -> `Nuketown 2020`).
fn map_title(map: &str) -> String {
    map.split(':')
        .next_back()
        .unwrap_or(map)
        .trim_start_matches("mp_")
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A friend's status as his row shows it.
pub(crate) fn status_text(f: Option<&FriendStatus>) -> String {
    match f {
        Some(f) if f.online => match &f.room {
            Some(r) if r.in_match => "IN A MATCH".to_owned(),
            Some(r) => format!("IN A LOBBY  {}/{}", r.players, r.max_players),
            None => "ONLINE".to_owned(),
        },
        _ => "OFFLINE".to_owned(),
    }
}

/// The screen's rows: ADD FRIEND (showing the code being typed), then his
/// friends (online first, then by name or code).
pub(crate) fn rows(shell: &Shell) -> Vec<Row> {
    let (_, mut list) = state();
    list.sort_by_key(|(code, f)| {
        let online = f.as_ref().is_some_and(|f| f.online);
        (!online, f.as_ref().map(|f| f.name.to_lowercase()).unwrap_or_default(), *code)
    });
    let mut out = vec![Row {
        label: "ADD FRIEND".to_owned(),
        sub: "Press Enter and type your friend's 8-digit code, then Enter to add them.".to_owned(),
        value: shell.code_entry.as_ref().map(|t| format!("{t}_")),
        act: Act::AddFriend,
    }];
    for (code, f) in list {
        let name = f.as_ref().filter(|f| !f.name.is_empty()).map_or_else(
            || format!("FRIEND {}", GameSettings::friend_code_text(code)),
            |f| f.name.clone(),
        );
        out.push(Row {
            label: name,
            sub: format!("Code {}  ·  Del removes them.", GameSettings::friend_code_text(code)),
            value: Some(status_text(f.as_ref())),
            act: Act::Friend(code),
        });
    }
    out
}

/// What Enter on a friend says.
pub(crate) fn friend_note(code: u32) -> String {
    let (_, list) = state();
    let f = list.into_iter().find(|(c, _)| *c == code).and_then(|(_, f)| f);
    let name = f.as_ref().filter(|f| !f.name.is_empty()).map_or_else(|| GameSettings::friend_code_text(code), |f| f.name.clone());
    match f {
        Some(f) if f.online => match &f.room {
            Some(r) => format!(
                "{name} is in a lobby on {} ({}). Joining friends' lobbies comes with online lobbies.",
                map_title(&r.map),
                r.mode.to_uppercase()
            ),
            None => format!("{name} is online."),
        },
        _ => format!("{name} is offline (or not on your server)."),
    }
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    header(c, cx, &["FRIENDS"]);
    c.spawn(Node { flex_direction: FlexDirection::Column, row_gap: cx.px(8.0), ..abs(Some(72.0), Some(176.0), None, None, cx) })
        .with_children(|col| menu_rows(col, cx, rows, shell.focus, 720.0, false));
    let (mine, list) = state();
    let focused = rows.get(shell.focus).map(|r| r.act);
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
        p.spawn((Text::new("YOUR FRIEND CODE"), cx.bold(22.0), TextColor(FAINT)));
        p.spawn((Text::new(GameSettings::friend_code_text(mine)), cx.bold(72.0), TextColor(WHITE)));
        p.spawn((
            Text::new("Give it to a friend so they can add you. You both need to play on the same server."),
            cx.regular(20.0),
            TextColor(DIM),
        ));
        p.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), margin: UiRect::vertical(cx.px(8.0)), ..default() }, BackgroundColor(cx.pal.line)));
        let online = list.iter().filter(|(_, f)| f.as_ref().is_some_and(|f| f.online)).count();
        p.spawn((Text::new(format!("{online} OF {} FRIENDS ONLINE", list.len())), cx.bold(24.0), TextColor(cx.pal.accent)));
        if let Some(Act::Friend(code)) = focused
            && let Some((_, Some(f))) = list.iter().find(|(c, _)| *c == code)
            && let Some(r) = f.room.as_ref().filter(|_| f.online)
        {
            p.spawn((Text::new(r.name.to_uppercase()), cx.bold(30.0), TextColor(WHITE)));
            p.spawn((
                Text::new(format!(
                    "{}  ·  {}  ·  {}/{} PLAYERS{}",
                    map_title(&r.map).to_uppercase(),
                    r.mode.to_uppercase(),
                    r.players,
                    r.max_players,
                    if r.in_match { "  ·  IN A MATCH" } else if r.locked { "  ·  CLOSED" } else { "" }
                )),
                cx.bold(21.0),
                TextColor(DIM),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn friend(code: u32, online: bool, room: Option<(bool, u8)>) -> FriendStatus {
        FriendStatus {
            code,
            online,
            name: format!("p{code}"),
            room: room.map(|(in_match, players)| master_protocol_advert(in_match, players)),
        }
    }

    fn master_protocol_advert(in_match: bool, players: u8) -> net::master_protocol::Advert {
        net::master_protocol::Advert {
            id: Default::default(),
            name: "lobby".to_owned(),
            map: "mp_raid".to_owned(),
            mode: "tdm".to_owned(),
            players,
            max_players: 12,
            locked: false,
            in_match,
            requires: Default::default(),
            generation: 0,
        }
    }

    #[test]
    fn statuses_read_as_the_rows_show_them() {
        assert_eq!(status_text(None), "OFFLINE");
        assert_eq!(status_text(Some(&friend(1, false, None))), "OFFLINE");
        assert_eq!(status_text(Some(&friend(1, true, None))), "ONLINE");
        assert_eq!(status_text(Some(&friend(1, true, Some((false, 3))))), "IN A LOBBY  3/12");
        assert_eq!(status_text(Some(&friend(1, true, Some((true, 9))))), "IN A MATCH");
    }

    #[test]
    fn map_names_read_as_titles() {
        assert_eq!(map_title("mp_nuketown_2020"), "Nuketown 2020");
        assert_eq!(map_title("iw4:mp_rust"), "Rust");
    }

    #[test]
    fn edits_add_and_remove_codes() {
        let mut s = GameSettings { friend_code: 11_111_111, ..GameSettings::default() };
        queue_add(22_222_222);
        queue_add(22_222_222);
        queue_add(11_111_111);
        queue_add(42);
        assert!(apply_edits(&mut s));
        assert_eq!(s.friends, vec![22_222_222]);
        queue_remove(22_222_222);
        assert!(apply_edits(&mut s));
        assert!(s.friends.is_empty());
    }
}
