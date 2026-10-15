//! bo2mp shell: WARZONE - Resurgence on Rebirth Island. Its lobby (squad
//! size, squads, bot difficulty) and START, which runs the free-for-all
//! with the Resurgence script on (`sim::set_resurgence`).
//!
//! Rebirth Island plays once its map is installed and listed;
//! `BO2_BR_DEV_MAP=<map>` plays the mode on another map until then.

use bevy::prelude::*;
use gamemode_iw4::resurgence;

use super::bridge::{self, Catalog, Request, StartSpec};
use super::theme::*;
use super::view::{Ctx, Kids, abs, header, menu_rows};
use super::{Act, DIFFICULTY, Row, Setting, Shell};

pub(crate) const SQUAD_NAMES: [&str; 4] = ["SOLOS", "DUOS", "TRIOS", "QUADS"];

/// Every player in the match fits in the server's 18.
pub(crate) const MAX_PLAYERS: u32 = 18;

pub(crate) fn max_squads(size: u8) -> u32 {
    (MAX_PLAYERS / u32::from(size.max(1))).max(2)
}

/// The map Resurgence runs on: Rebirth Island, else the dev map.
pub(crate) fn map(c: &Catalog) -> Result<(String, String), String> {
    if let Some(m) = c.map(resurgence::MAP) {
        return Ok((m.ui_name.clone(), m.title.clone()));
    }
    let dev = std::env::var("BO2_BR_DEV_MAP")
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());
    match dev {
        Some(d) => match c.map(&d) {
            Some(m) => Ok((m.ui_name.clone(), format!("{} (DEV)", m.title))),
            None => Err(format!("BO2_BR_DEV_MAP: no map `{d}` installed.")),
        },
        None => Err("Rebirth Island is not ported yet. Set BO2_BR_DEV_MAP=<map> to play Resurgence on another map.".to_owned()),
    }
}

pub(crate) fn rows(s: &Shell, c: &Catalog) -> Vec<Row> {
    let mut r = vec![
        Row::new(
            "START RESURGENCE",
            "Drop in with your squad. Last squad standing wins.",
            Act::BrStart,
        ),
        Row::new("SQUAD SIZE", "", Act::Cycle(Setting::BrSquadSize)),
        Row::new(
            "SQUADS",
            "Your squad and the bot squads, filled with bots.",
            Act::Cycle(Setting::BrSquads),
        ),
        Row::new("BOT DIFFICULTY", "", Act::Cycle(Setting::Difficulty)),
        Row::new("MAP", "", Act::Info),
    ];
    let size = s.br_squad_size.clamp(1, resurgence::MAX_SQUAD_SIZE);
    r[1].value = Some(SQUAD_NAMES[usize::from(size - 1)].to_owned());
    r[2].value = Some(s.br_squads.min(max_squads(size)).to_string());
    r[3].value = Some(DIFFICULTY[s.difficulty.min(3) as usize].to_owned());
    match map(c) {
        Ok((_, title)) => r[4].value = Some(title.to_uppercase()),
        Err(e) => {
            r[4].value = Some(resurgence::MAP_TITLE.to_owned());
            r[4].sub = e;
        }
    }
    r
}

pub(crate) fn cycle(s: &mut Shell, what: Setting, by: i32) {
    match what {
        Setting::BrSquadSize => {
            let n = i32::from(resurgence::MAX_SQUAD_SIZE);
            s.br_squad_size = ((i32::from(s.br_squad_size) - 1 + by).rem_euclid(n) + 1) as u8;
            s.br_squads = s.br_squads.min(max_squads(s.br_squad_size));
        }
        Setting::BrSquads => {
            let max = max_squads(s.br_squad_size) as i32;
            s.br_squads = ((s.br_squads as i32 - 2 + by).rem_euclid(max - 1) + 2) as u32;
        }
        _ => {}
    }
}

/// START: the free-for-all with the squads' bots, Resurgence on.
pub(crate) fn start(s: &mut Shell, c: &Catalog) -> Result<(), String> {
    if bridge::starting_in().is_some() {
        return Ok(());
    }
    let (map, _) = map(c)?;
    let size = s.br_squad_size.clamp(1, resurgence::MAX_SQUAD_SIZE);
    let squads = s.br_squads.clamp(2, max_squads(size));
    s.played = true;
    sim::set_moab_kills(0);
    bridge::request(Request::Start(StartSpec {
        map,
        gametype: resurgence::BASE_GAMETYPE.to_owned(),
        bot_friends: u32::from(size) - 1,
        bot_enemies: (squads - 1) * u32::from(size),
        bot_difficulty: s.difficulty,
        public: false,
        br: Some(size),
    }));
    Ok(())
}

pub(crate) fn view(c: &mut Kids, cx: &mut Ctx, shell: &Shell, rows: &[Row]) {
    header(c, cx, &["WARZONE", "RESURGENCE"]);
    c.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: cx.px(8.0),
        ..abs(Some(72.0), Some(176.0), None, None, cx)
    })
    .with_children(|col| menu_rows(col, cx, rows, shell.focus, 640.0, false));
    let r = resurgence::REBIRTH;
    let redeploys: Vec<String> = r
        .phases
        .iter()
        .map(|p| {
            p.redeploy_s
                .map_or_else(|| "OFF".to_owned(), |s| format!("{s}s"))
        })
        .collect();
    c.spawn((
        Node {
            width: cx.px(760.0),
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(12.0),
            padding: UiRect::all(cx.px(28.0)),
            border: UiRect::top(cx.px(3.0)),
            ..abs(Some(780.0), Some(176.0), None, None, cx)
        },
        BackgroundColor(cx.pal.panel),
        BorderColor::all(cx.pal.accent),
    ))
    .with_children(|p| {
        p.spawn((Text::new(resurgence::MAP_TITLE), cx.bold(52.0), TextColor(WHITE)));
        for line in [
            "While a squadmate lives, you redeploy after a wait that grows as the circle closes. A squad with nobody alive is out.".to_owned(),
            format!("{} circles. Redeploy wait by circle: {}.", r.phases.len(), redeploys.join(" / ")),
            format!("Armor: up to {} plates of {}. Hold [Use] to put one in.", r.plate_slots, r.plate_hp),
            "Cash from kills and the ground buys a loadout drop, a self-revive kit, a UAV, plates or a squadmate back at the buy stations.".to_owned(),
        ] {
            p.spawn((Text::new(line), cx.regular(22.0), TextColor(TEXT)));
        }
    });
}
