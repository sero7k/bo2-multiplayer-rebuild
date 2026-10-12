//! bo2mp shell: calling cards and emblems. Black Ops II's banners and
//! emblems are gone from the shell; every player card shows a Modern
//! Warfare 2 or 3 calling card (`cardtitle_*`) and emblem (`cardicon_*`),
//! read from that game's own files (`iw4:<material>` / `iw5:<material>`
//! through the shell's pictures). His are chosen in Barracks and kept in his
//! stats file; a bot's follow from its name, so it keeps them.
//!
//! The lists are the games' own: every `cardtitle_*` / `cardicon_*` image
//! their archives hold, read once off the main thread. Until they are in
//! (or with neither game installed) a short list of Modern Warfare 2's
//! names stands in.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// His choices in his stats file (`s:iw4:cardtitle_...`; none = the
/// default).
pub(crate) const CARD_STAT: &str = "bo2mp.callingcard";
pub(crate) const EMBLEM_STAT: &str = "bo2mp.emblem";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Kind {
    Card,
    Emblem,
}

impl Kind {
    fn stem(self) -> &'static str {
        match self {
            Kind::Card => "cardtitle_",
            Kind::Emblem => "cardicon_",
        }
    }

    pub(crate) fn stat(self) -> &'static str {
        match self {
            Kind::Card => CARD_STAT,
            Kind::Emblem => EMBLEM_STAT,
        }
    }
}

/// Modern Warfare 2's names that stand in while the lists are read (the
/// first is the default when the games' lists lack the preferred ones).
const STAND_IN_CARDS: [&str; 6] = [
    "iw4:cardtitle_tf141",
    "iw4:cardtitle_bloodsplat",
    "iw4:cardtitle_army_ranger",
    "iw4:cardtitle_seals",
    "iw4:cardtitle_ac130",
    "iw4:cardtitle_harrier",
];
const STAND_IN_EMBLEMS: [&str; 6] = [
    "iw4:cardicon_tf141",
    "iw4:cardicon_army",
    "iw4:cardicon_seal",
    "iw4:cardicon_skull_black",
    "iw4:cardicon_ac130",
    "iw4:cardicon_harrier",
];

#[derive(Default)]
struct Lists {
    cards: Vec<String>,
    emblems: Vec<String>,
}

static LISTS: Mutex<Option<Arc<Lists>>> = Mutex::new(None);
static STARTED: AtomicBool = AtomicBool::new(false);

/// Every name of a kind in the games' files: Modern Warfare 2's, then 3's
/// (`iw4:` / `iw5:`). Prestige icons are left out (they are rank icons).
fn read_lists() -> Lists {
    let mut lists = Lists::default();
    for game in ["iw4", "iw5"] {
        for (kind, out) in [(Kind::Card, &mut lists.cards), (Kind::Emblem, &mut lists.emblems)] {
            out.extend(
                crate::foreign_maps::game_image_names(game, kind.stem())
                    .into_iter()
                    .filter(|n| !n.contains("prestige"))
                    .map(|n| format!("{game}:{n}")),
            );
        }
    }
    diag::info!(Ui, "bo2mp shell: {} calling cards and {} emblems in Modern Warfare 2 and 3", lists.cards.len(), lists.emblems.len());
    lists
}

/// Starts reading the lists (once a run).
pub(crate) fn warm() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("ui-calling-cards".into())
        .spawn(|| {
            let lists = read_lists();
            if let Ok(mut l) = LISTS.lock() {
                l.get_or_insert_with(|| Arc::new(lists));
            }
        })
        .ok();
}

/// The preview's lists (no games' files there).
pub(crate) fn set_preview(cards: Vec<String>, emblems: Vec<String>) {
    STARTED.store(true, Ordering::SeqCst);
    if let Ok(mut l) = LISTS.lock() {
        *l = Some(Arc::new(Lists { cards, emblems }));
    }
}

/// The names of a kind there are to pick from (the stand-ins until the
/// games' lists are in, or when they are empty).
pub(crate) fn list(kind: Kind) -> Vec<String> {
    warm();
    let read = LISTS.lock().ok().and_then(|l| l.clone());
    let found = read.map(|l| match kind {
        Kind::Card => l.cards.clone(),
        Kind::Emblem => l.emblems.clone(),
    });
    match found {
        Some(v) if !v.is_empty() => v,
        _ => stand_ins(kind),
    }
}

fn stand_ins(kind: Kind) -> Vec<String> {
    match kind {
        Kind::Card => STAND_IN_CARDS.iter().map(|s| (*s).to_owned()).collect(),
        Kind::Emblem => STAND_IN_EMBLEMS.iter().map(|s| (*s).to_owned()).collect(),
    }
}

/// The default of a kind in `list`: the first stand-in it has, else its
/// first Modern Warfare 2 name, else its first.
pub(crate) fn default_in(kind: Kind, list: &[String]) -> String {
    stand_ins(kind)
        .into_iter()
        .find(|s| list.contains(s))
        .or_else(|| list.iter().find(|n| n.starts_with("iw4:")).cloned())
        .or_else(|| list.first().cloned())
        .unwrap_or_else(|| stand_ins(kind).swap_remove(0))
}

/// His card or emblem: the one his stats hold when it is in `list`, else
/// the default.
pub(crate) fn chosen_in(kind: Kind, stat: &str, list: &[String]) -> String {
    let stat = stat.trim();
    if !stat.is_empty() && list.iter().any(|n| n == stat) {
        stat.to_owned()
    } else {
        default_in(kind, list)
    }
}

pub(crate) fn chosen(kind: Kind, stat: &str) -> String {
    chosen_in(kind, stat, &list(kind))
}

/// The next (`by` +1) or previous (-1) name after `current` in `list`.
pub(crate) fn step_in(list: &[String], current: &str, by: i32) -> Option<String> {
    if list.is_empty() {
        return None;
    }
    let at = list.iter().position(|n| n == current).map_or(if by > 0 { -1 } else { 0 }, |i| i as i32);
    Some(list[(at + by).rem_euclid(list.len() as i32) as usize].clone())
}

/// `current`'s place in `list` (1-based) and how many there are.
pub(crate) fn place_in(list: &[String], current: &str) -> (usize, usize) {
    (list.iter().position(|n| n == current).map_or(0, |i| i + 1), list.len())
}

/// What he picked this session, until his stats (read back every few
/// frames) say the same.
static PICKED: Mutex<[Option<String>; 2]> = Mutex::new([None, None]);

fn slot(kind: Kind) -> usize {
    match kind {
        Kind::Card => 0,
        Kind::Emblem => 1,
    }
}

/// His card or emblem: the one he just picked, else his stats'.
pub(crate) fn mine(kind: Kind, stat: &str) -> String {
    let picked = PICKED.lock().ok().and_then(|mut p| {
        let at = &mut p[slot(kind)];
        if at.as_deref() == Some(stat.trim()) {
            *at = None;
        }
        at.clone()
    });
    picked.unwrap_or_else(|| chosen(kind, stat))
}

/// He picks the next (`by` +1) or previous (-1) one: kept in his stats.
pub(crate) fn pick(kind: Kind, stat: &str, by: i32) {
    let now = mine(kind, stat);
    if let Some(next) = step_in(&list(kind), &now, by) {
        if let Ok(mut p) = PICKED.lock() {
            p[slot(kind)] = Some(next.clone());
        }
        super::bridge::request(super::bridge::Request::SetLook(kind, next));
    }
}

/// FNV-1a: a name's number, the same every run.
fn hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3))
}

/// A bot's card or emblem: picked by its name, the same every time.
pub(crate) fn for_name_in(kind: Kind, name: &str, list: &[String]) -> String {
    if list.is_empty() {
        return default_in(kind, list);
    }
    let salt = match kind {
        Kind::Card => "card:",
        Kind::Emblem => "emblem:",
    };
    list[(hash(&format!("{salt}{}", name.trim().to_ascii_lowercase())) % list.len() as u64) as usize].clone()
}

pub(crate) fn for_name(kind: Kind, name: &str) -> String {
    for_name_in(kind, name, &list(kind))
}

/// A name as he reads it: `iw4:cardtitle_blood_splat` -> `MW2 · BLOOD SPLAT`.
pub(crate) fn label(name: &str) -> String {
    let (game, material) = name.split_once(':').unwrap_or(("", name));
    let game = match game {
        "iw4" => "MW2",
        "iw5" => "MW3",
        _ => "",
    };
    let rest = material.trim_start_matches("cardtitle_").trim_start_matches("cardicon_").replace('_', " ").to_uppercase();
    if game.is_empty() { rest } else { format!("{game} · {rest}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn labels_read_well() {
        assert_eq!(label("iw4:cardtitle_blood_splat"), "MW2 · BLOOD SPLAT");
        assert_eq!(label("iw5:cardicon_skull"), "MW3 · SKULL");
    }

    #[test]
    fn bots_keep_theirs() {
        let l = names(&["iw4:cardtitle_a", "iw4:cardtitle_b", "iw5:cardtitle_c", "iw5:cardtitle_d"]);
        let a = for_name_in(Kind::Card, "Bot 3", &l);
        assert_eq!(a, for_name_in(Kind::Card, "Bot 3", &l));
        assert_eq!(a, for_name_in(Kind::Card, " bot 3 ", &l));
        assert!(l.contains(&a));
        // (Not every bot the same.)
        let picks: std::collections::HashSet<String> = (1..=17).map(|i| for_name_in(Kind::Card, &format!("Bot {i}"), &l)).collect();
        assert!(picks.len() > 1);
    }

    #[test]
    fn his_choice_or_the_default() {
        let l = names(&["iw5:cardtitle_x", "iw4:cardtitle_bloodsplat", "iw4:cardtitle_y"]);
        assert_eq!(chosen_in(Kind::Card, "", &l), "iw4:cardtitle_bloodsplat");
        assert_eq!(chosen_in(Kind::Card, "iw5:cardtitle_x", &l), "iw5:cardtitle_x");
        // (One the games no longer have: the default.)
        assert_eq!(chosen_in(Kind::Card, "iw4:cardtitle_gone", &l), "iw4:cardtitle_bloodsplat");
        // (No preferred one: the first Modern Warfare 2 name.)
        let l = names(&["iw5:cardicon_a", "iw4:cardicon_b"]);
        assert_eq!(default_in(Kind::Emblem, &l), "iw4:cardicon_b");
    }

    #[test]
    fn stepping_wraps() {
        let l = names(&["a", "b", "c"]);
        assert_eq!(step_in(&l, "a", 1).as_deref(), Some("b"));
        assert_eq!(step_in(&l, "c", 1).as_deref(), Some("a"));
        assert_eq!(step_in(&l, "a", -1).as_deref(), Some("c"));
        assert_eq!(step_in(&l, "zz", 1).as_deref(), Some("a"));
        assert_eq!(step_in(&[], "a", 1), None);
        assert_eq!(place_in(&l, "b"), (2, 3));
    }
}
