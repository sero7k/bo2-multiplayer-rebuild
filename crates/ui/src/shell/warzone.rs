//! bo2mp shell: WARZONE - Multiplayer's menu again, for Resurgence on
//! Rebirth Island: its menu (QUICK PLAY, PRIVATE MATCH, Create-a-Class,
//! Barracks, Options), the private match's squad lobby (SQUAD / LOADOUTS /
//! SETTINGS: the mode, squad size, squads, bot difficulty) and the mode
//! picker. What the rows do is here; their look is `wz_view`, the colours
//! and words `wz_theme`. DEPLOY runs the free-for-all with the Resurgence
//! script on (`sim::set_resurgence`).
//!
//! Rebirth Island plays once its map is installed and listed;
//! `BO2_BR_DEV_MAP=<map>` plays the mode on another map until then.

use gamemode_iw4::resurgence;

use super::bridge::{self, Catalog, Request, StartSpec};
use super::wz_theme as wt;
use super::{Act, DIFFICULTY, Row, Screen, Setting, Shell, Tab};

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

/// A WARZONE playlist: its name, what it is, and the squad size it sets
/// (`None`: the size picked in the lobby).
pub(crate) struct WzMode {
    pub title: &'static str,
    pub desc: &'static str,
    pub squad: Option<u8>,
}

/// The playlists, Resurgence on Rebirth Island first (the main one).
pub(crate) const MODES: [WzMode; 5] = [
    WzMode {
        title: "RESURGENCE — REBIRTH ISLAND",
        desc: "Squads drop onto Rebirth Island. While a squadmate lives you redeploy; the last squad standing wins. Keeps the squad size you picked.",
        squad: None,
    },
    WzMode { title: "RESURGENCE SOLOS", desc: "Resurgence alone: you redeploy until the redeploys switch off.", squad: Some(1) },
    WzMode { title: "RESURGENCE DUOS", desc: "Resurgence in squads of two.", squad: Some(2) },
    WzMode { title: "RESURGENCE TRIOS", desc: "Resurgence in squads of three.", squad: Some(3) },
    WzMode { title: "RESURGENCE QUADS", desc: "Resurgence in squads of four.", squad: Some(4) },
];

/// The squad size played (1-4).
pub(crate) fn squad_size(s: &Shell) -> u8 {
    s.br_squad_size.clamp(1, resurgence::MAX_SQUAD_SIZE)
}

/// The squads in the match (2 up to what fits in 18).
pub(crate) fn squads(s: &Shell) -> u32 {
    s.br_squads.clamp(2, max_squads(squad_size(s)))
}

/// The mode as the lobby shows it (`RESURGENCE TRIOS`).
pub(crate) fn mode_line(s: &Shell) -> String {
    format!("{} {}", resurgence::DISPLAY_NAME, SQUAD_NAMES[usize::from(squad_size(s) - 1)])
}

fn row(t: (&str, &str), act: Act) -> Row {
    Row::new(t.0, t.1, act)
}

/// The MAP row: Rebirth Island (or the dev map), or why it cannot play.
fn map_row(c: &Catalog) -> Row {
    let mut r = Row::new(wt::MAP, "", Act::Info);
    match map(c) {
        Ok((_, title)) => r.value = Some(title.to_uppercase()),
        Err(e) => {
            r.value = Some(resurgence::MAP_TITLE.to_owned());
            r.sub = e;
        }
    }
    r
}

/// The rows of the WARZONE screen he is on.
pub(crate) fn rows(s: &Shell, c: &Catalog) -> Vec<Row> {
    match s.screen {
        Screen::WzMenu => vec![
            row(wt::QUICK_PLAY, Act::WzQuickPlay),
            row(wt::PRIVATE_MATCH, Act::WzPrivate),
            row(wt::LOADOUTS, Act::Classes),
            row(wt::BARRACKS, Act::Barracks),
            row(wt::OPTIONS, Act::Options),
        ],
        Screen::WzModePick => MODES.iter().enumerate().map(|(i, m)| Row::new(m.title, m.desc, Act::Choose(i))).collect(),
        _ => match s.tab {
            Tab::Lobby => vec![row(wt::DEPLOY, Act::BrStart), row(wt::CHANGE_MODE, Act::WzPickMode), map_row(c)],
            Tab::Loadouts => vec![row(wt::LOADOUTS, Act::Classes), row(wt::BARRACKS, Act::Barracks)],
            Tab::Settings => {
                let mut r = vec![
                    Row::new(wt::GAME_MODE, "", Act::Cycle(Setting::BrMode)),
                    Row::new(wt::SQUAD_SIZE, "", Act::Cycle(Setting::BrSquadSize)),
                    row(wt::SQUADS, Act::Cycle(Setting::BrSquads)),
                    Row::new(wt::BOT_DIFFICULTY, "", Act::Cycle(Setting::Difficulty)),
                    map_row(c),
                ];
                r[0].value = Some(MODES[s.wz_mode.min(MODES.len() - 1)].title.to_owned());
                r[1].value = Some(SQUAD_NAMES[usize::from(squad_size(s) - 1)].to_owned());
                r[2].value = Some(squads(s).to_string());
                r[3].value = Some(DIFFICULTY[s.difficulty.min(3) as usize].to_owned());
                r
            }
        },
    }
}

/// Picks playlist `i`, and its squad size if it sets one.
pub(crate) fn set_mode(s: &mut Shell, i: usize) {
    let i = i.min(MODES.len() - 1);
    s.wz_mode = i;
    if let Some(n) = MODES[i].squad {
        s.br_squad_size = n;
        s.br_squads = s.br_squads.min(max_squads(n));
    }
}

pub(crate) fn cycle(s: &mut Shell, what: Setting, by: i32) {
    match what {
        Setting::BrSquadSize => {
            let n = i32::from(resurgence::MAX_SQUAD_SIZE);
            s.br_squad_size = ((i32::from(s.br_squad_size) - 1 + by).rem_euclid(n) + 1) as u8;
            s.br_squads = s.br_squads.min(max_squads(s.br_squad_size));
            // (A size picked by hand: the main playlist, which keeps it.)
            s.wz_mode = 0;
        }
        Setting::BrSquads => {
            let max = max_squads(s.br_squad_size) as i32;
            s.br_squads = ((s.br_squads as i32 - 2 + by).rem_euclid(max - 1) + 2) as u32;
        }
        Setting::BrMode => set_mode(s, (s.wz_mode as i32 + by).rem_euclid(MODES.len() as i32) as usize),
        _ => {}
    }
}

/// DEPLOY: the free-for-all with the squads' bots, Resurgence on.
pub(crate) fn start(s: &mut Shell, c: &Catalog) -> Result<(), String> {
    if bridge::starting_in().is_some() {
        return Ok(());
    }
    let (map, _) = map(c)?;
    let (size, squads) = (squad_size(s), squads(s));
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
