//! bo2mp shell: Zombies' own record and rank, kept apart from
//! Multiplayer's (its own file, `iw4l-artifacts/bo2zm_record.txt`).
//!
//! The rank is Black Ops II Zombies': five symbols (`menu_zm_rank_1..5`:
//! the tally, the shotgun shells, the skull, the skull and knife, the skull
//! and shotguns), each in five steps, earned from the same things BO2's
//! hidden rank weighed - rounds reached, kills, headshots and revives, less
//! for downs and for leaving a game early. Like BO2's it can go down: a run
//! of short games and downs loses ground, and the symbols from the skull up
//! show their worn (`_ded`) version after a game that went badly. Treyarch
//! never published BO2's exact formula; the weights here follow the
//! metrics it was known to use.
//!
//! A game is counted when it ends (the game over) or when he leaves it,
//! from the scoreboard the game keeps (`bo2zm_scoreboard`) and its round.

use std::sync::Mutex;

use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

const PATH: &str = "iw4l-artifacts/bo2zm_record.txt";

/// Rank points for each of the 25 steps (5 symbols by 5 steps).
const STEPS: usize = 25;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Record {
    pub games: u32,
    pub rounds: u32,
    pub best_round: u32,
    pub kills: u32,
    pub headshots: u32,
    pub downs: u32,
    pub revives: u32,
    pub points: u64,
    /// The rank's points (never below 0).
    pub rank_points: f32,
    /// The last game went badly (a down-heavy or short game).
    pub slump: bool,
}

static RECORD: Mutex<Option<Record>> = Mutex::new(None);

fn parse(text: &str) -> Record {
    let mut r = Record::default();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim(), v.trim());
        let n = || v.parse::<u32>().unwrap_or(0);
        match k {
            "games" => r.games = n(),
            "rounds" => r.rounds = n(),
            "best_round" => r.best_round = n(),
            "kills" => r.kills = n(),
            "headshots" => r.headshots = n(),
            "downs" => r.downs = n(),
            "revives" => r.revives = n(),
            "points" => r.points = v.parse().unwrap_or(0),
            "rank_points" => r.rank_points = v.parse().unwrap_or(0.0),
            "slump" => r.slump = v == "1",
            _ => {}
        }
    }
    r
}

fn text(r: &Record) -> String {
    format!(
        "games={}\nrounds={}\nbest_round={}\nkills={}\nheadshots={}\ndowns={}\nrevives={}\npoints={}\nrank_points={:.1}\nslump={}\n",
        r.games,
        r.rounds,
        r.best_round,
        r.kills,
        r.headshots,
        r.downs,
        r.revives,
        r.points,
        r.rank_points,
        u8::from(r.slump)
    )
}

/// His record (read once a run).
pub(crate) fn get() -> Record {
    let mut g = RECORD.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.get_or_insert_with(|| std::fs::read_to_string(PATH).map(|t| parse(&t)).unwrap_or_default()).clone()
}

/// One game's numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Game {
    pub round: u32,
    pub score: u32,
    pub kills: u32,
    pub downs: u32,
    pub revives: u32,
    pub headshots: u32,
    /// He left before it ended.
    pub left: bool,
}

/// What a game is worth to the rank (can be below 0).
pub(crate) fn game_value(g: &Game) -> f32 {
    let mut v = g.round as f32 * 10.0 + g.kills as f32 * 0.5 + g.headshots as f32 * 0.5 + g.revives as f32 * 15.0 - g.downs as f32 * 12.0;
    if g.left && g.round < 5 {
        v -= 60.0;
    }
    v
}

/// The points step `n` (1-based) begins at: each step costs more.
fn step_floor(n: usize) -> f32 {
    if n <= 1 {
        return 0.0;
    }
    let k = (n - 1) as f32;
    120.0 * k + 18.0 * k * k
}

/// A game counted.
pub(crate) fn add(g: Game) {
    let mut r = get();
    r.games += 1;
    r.rounds += g.round;
    r.best_round = r.best_round.max(g.round);
    r.kills += g.kills;
    r.headshots += g.headshots;
    r.downs += g.downs;
    r.revives += g.revives;
    r.points += u64::from(g.score);
    let v = game_value(&g);
    r.rank_points = (r.rank_points + v).max(0.0);
    r.slump = v < 0.0 || (g.left && g.round < 5);
    diag::info!(Ui, "bo2mp zombies: game counted {g:?}: rank points {v:+.0} -> {:.0}", r.rank_points);
    let _ = std::fs::create_dir_all("iw4l-artifacts");
    if let Err(e) = std::fs::write(PATH, text(&r)) {
        diag::warn!(Ui, "bo2mp zombies: record not saved: {e}");
    }
    *RECORD.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
}

/// His rank: the symbol (1-5), its step (1-5), and how far to the next.
pub(crate) fn rank(r: &Record) -> (usize, usize, f32) {
    let n = (1..=STEPS).rev().find(|&n| r.rank_points >= step_floor(n)).unwrap_or(1);
    let frac = if n == STEPS {
        1.0
    } else {
        ((r.rank_points - step_floor(n)) / (step_floor(n + 1) - step_floor(n))).clamp(0.0, 1.0)
    };
    ((n - 1) / 5 + 1, (n - 1) % 5 + 1, frac)
}

/// The rank's symbol (BO2's `menu_zm_rank_<n>`, worn after a bad game from
/// the skull up).
pub(crate) fn icon(r: &Record) -> String {
    let (sym, _, _) = rank(r);
    if r.slump && sym >= 3 { format!("menu_zm_rank_{sym}_ded") } else { format!("menu_zm_rank_{sym}") }
}

pub(crate) fn rank_title(sym: usize) -> &'static str {
    match sym {
        1 => "TALLY",
        2 => "SHELLS",
        3 => "SKULL",
        4 => "SKULL AND KNIFE",
        _ => "SKULL AND SHOTGUNS",
    }
}

/// The game being played, as last seen.
#[derive(Default)]
pub(crate) struct Watch {
    last: Option<Game>,
    counted: bool,
    zone: String,
}

/// His row of the game's scoreboard and its round.
fn current(presented: &PresentedSnapshot, local: &LocalPresentClient) -> Option<(Game, bool)> {
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let round: u32 = dvars.string("bo2zm_round")?.trim().parse().ok()?;
    let board = dvars.string("bo2zm_scoreboard").unwrap_or_default();
    let me = local.0.0;
    let row = board.split(';').find(|r| r.split(':').next().and_then(|c| c.parse::<u32>().ok()) == Some(me))?;
    let cells: Vec<&str> = row.split(':').collect();
    let n = |i: usize| cells.get(i).and_then(|c| c.trim().parse().ok()).unwrap_or(0);
    let over = presented.player(local.0).is_some_and(|p| p.pm_type == playerstate_iw4::PM_TYPE_INTERMISSION);
    Some((Game { round, score: n(2), kills: n(3), downs: n(4), revives: n(5), headshots: n(6), left: false }, over))
}

/// Counts each Zombies game once: at its game over, or as he leaves it
/// (the game goes away before it is over).
pub(crate) fn watch_games(
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    identity: Option<Res<frame::LaunchIdentity>>,
    mut st: Local<Watch>,
) {
    let zone = identity.as_deref().map(|i| i.zone.clone()).unwrap_or_default();
    let zm = crate::lui_hud::is_zombies_zone(&zone);
    let now = match (zm, presented.as_deref(), local.as_deref()) {
        (true, Some(p), Some(l)) => current(p, l),
        _ => None,
    };
    match now {
        Some((game, over)) => {
            if zone != st.zone {
                st.zone.clone_from(&zone);
                st.counted = false;
            }
            // (A restart starts a new game: round 1 again.)
            if st.counted && !over && st.last.is_some_and(|l| game.round < l.round) {
                st.counted = false;
            }
            if over && !st.counted {
                st.counted = true;
                add(game);
            }
            st.last = Some(game);
        }
        None => {
            // Out of the game before its end: counted as left.
            if let Some(mut last) = st.last.take()
                && !st.counted
                && last.round > 0
            {
                last.left = true;
                add(last);
            }
            st.counted = false;
            st.zone.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_rise_and_rank_starts_at_one() {
        assert_eq!(rank(&Record::default()), (1, 1, 0.0));
        let mut r = Record { rank_points: step_floor(6), ..Default::default() };
        assert_eq!(rank(&r).0, 2);
        assert_eq!(rank(&r).1, 1);
        r.rank_points = 1e9;
        assert_eq!((rank(&r).0, rank(&r).1), (5, 5));
        assert!((2..STEPS).all(|n| step_floor(n + 1) - step_floor(n) > step_floor(n) - step_floor(n - 1)));
    }

    #[test]
    fn leaving_early_costs() {
        let g = Game { round: 2, kills: 10, ..Default::default() };
        assert!(game_value(&Game { left: true, ..g }) < 0.0);
        assert!(game_value(&g) > 0.0);
    }

    #[test]
    fn the_file_reads_back() {
        let r = Record { games: 3, best_round: 31, rank_points: 512.5, slump: true, ..Default::default() };
        assert_eq!(parse(&text(&r)), r);
    }
}
