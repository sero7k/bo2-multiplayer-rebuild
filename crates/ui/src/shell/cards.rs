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
//!
//! Modern Warfare III's (2023) emblems and calling cards come after them
//! (`jup:<name>`, `crate::jup_cards`), many of them flipbooks. The pickers
//! show one game at a time, its own category ([`GAMES`]).

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
    let (cards, emblems) = crate::jup_cards::names();
    lists.cards.extend(cards);
    lists.emblems.extend(emblems);
    lists
}

/// The games the pickers group them by, in order: (prefix, tab).
pub(crate) const GAMES: [(&str, &str); 3] =
    [("iw4", "MODERN WARFARE 2"), ("iw5", "MODERN WARFARE 3"), (crate::jup_cards::PREFIX, "MODERN WARFARE III")];

fn game_of(name: &str) -> &str {
    name.split_once(':').map_or("", |(g, _)| g)
}

/// The categories `list` has: indices into [`GAMES`], in order.
pub(crate) fn categories_in(list: &[String]) -> Vec<usize> {
    (0..GAMES.len()).filter(|&g| list.iter().any(|n| game_of(n) == GAMES[g].0)).collect()
}

/// The `cat`-th category of `list` (the last when past it): its names.
pub(crate) fn in_category(list: &[String], cat: usize) -> Vec<String> {
    let cats = categories_in(list);
    let Some(&g) = cats.get(cat.min(cats.len().saturating_sub(1))) else { return list.to_vec() };
    list.iter().filter(|n| game_of(n) == GAMES[g].0).cloned().collect()
}

/// The category of the picker a name is in (0 when none has it).
pub(crate) fn category_of_in(list: &[String], name: &str) -> usize {
    categories_in(list).iter().position(|&g| GAMES[g].0 == game_of(name)).unwrap_or(0)
}

/// What a picker shows of a kind: category `cat`'s names.
pub(crate) fn shown(kind: Kind, cat: usize) -> Vec<String> {
    in_category(&list(kind), cat)
}

/// The picker's category tabs of a kind.
pub(crate) fn category_tabs(kind: Kind) -> Vec<String> {
    categories_in(&list(kind)).iter().map(|&g| GAMES[g].1.to_owned()).collect()
}

/// How many frames a name's picture has (1: a still one, or not read yet).
pub(crate) fn frames(name: &str) -> u32 {
    if game_of(name) == crate::jup_cards::PREFIX { crate::jup_cards::frames(name) } else { 1 }
}

/// The shell name of a frame of a flipbook.
pub(crate) fn frame_name(name: &str, frame: u32) -> String {
    crate::jup_cards::frame_name(name, frame)
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

/// He picks one (Barracks' picker): kept in his stats.
pub(crate) fn set(kind: Kind, name: &str) {
    if let Ok(mut p) = PICKED.lock() {
        p[slot(kind)] = Some(name.to_owned());
    }
    super::bridge::request(super::bridge::Request::SetLook(kind, name.to_owned()));
}

/// The pickers' pages, as Modern Warfare 2's: emblems 8 across and 6 down,
/// calling cards in 3 columns of 9.
pub(crate) fn columns(kind: Kind) -> usize {
    match kind {
        Kind::Card => 3,
        Kind::Emblem => 8,
    }
}

pub(crate) fn per_page(kind: Kind) -> usize {
    match kind {
        Kind::Card => 27,
        Kind::Emblem => 48,
    }
}

/// How many pages `n` of a kind fill (at least one).
pub(crate) fn pages(kind: Kind, n: usize) -> usize {
    n.div_ceil(per_page(kind)).max(1)
}

/// The names on page `page` (the last page when past it).
pub(crate) fn page_items(kind: Kind, list: &[String], page: usize) -> Vec<String> {
    let per = per_page(kind);
    let page = page.min(pages(kind, list.len()) - 1);
    list.iter().skip(page * per).take(per).cloned().collect()
}

/// Words the material names run together or shorten, as he reads them.
const WORDS: [(&str, &str); 12] = [
    ("bloodsplat", "Blood Splat"),
    ("ac130", "AC-130"),
    ("tf141", "Task Force 141"),
    ("seals", "Navy SEALs"),
    ("seal", "Navy SEAL"),
    ("op", "Op"),
    ("mp", "MP"),
    ("cod", "CoD"),
    ("ffa", "FFA"),
    ("uav", "UAV"),
    ("emp", "EMP"),
    ("rpg", "RPG"),
];

/// A calling card's or emblem's name to show: Modern Warfare 2 and 3 keep
/// their titles' text in their string tables and localized zones, which the
/// shell does not read; the name comes from the material's
/// (`iw4:cardtitle_army_ranger` -> `Army Ranger`, `cardicon_1st_place` ->
/// `1st Place`).
pub(crate) fn title(name: &str) -> String {
    if game_of(name) == crate::jup_cards::PREFIX {
        return crate::jup_cards::title(name);
    }
    let material = name.split_once(':').map_or(name, |(_, m)| m);
    let rest = material.trim_start_matches("cardtitle_").trim_start_matches("cardicon_");
    let words: Vec<String> = rest
        .split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            if let Some((_, nice)) = WORDS.iter().find(|(k, _)| *k == w) {
                return (*nice).to_owned();
            }
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect();
    if words.is_empty() { material.to_owned() } else { words.join(" ") }
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
        "jup" => "MWIII",
        _ => "",
    };
    if game == "MWIII" {
        return format!("{game} · {}", title(name).to_uppercase());
    }
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
        assert_eq!(title("iw4:cardtitle_army_ranger"), "Army Ranger");
        assert_eq!(title("iw4:cardtitle_bloodsplat"), "Blood Splat");
        assert_eq!(title("iw5:cardicon_1st_place"), "1st Place");
        assert_eq!(title("iw4:cardicon_ac130"), "AC-130");
    }

    #[test]
    fn picker_pages() {
        let l: Vec<String> = (0..100).map(|i| format!("iw4:cardicon_{i}")).collect();
        assert_eq!(pages(Kind::Emblem, l.len()), 3);
        assert_eq!(page_items(Kind::Emblem, &l, 0).len(), 48);
        assert_eq!(page_items(Kind::Emblem, &l, 2).len(), 4);
        // (Past the last page: the last.)
        assert_eq!(page_items(Kind::Emblem, &l, 9)[0], "iw4:cardicon_96");
        assert_eq!(pages(Kind::Card, 0), 1);
        assert_eq!(page_items(Kind::Card, &l, 1).len(), 27);
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
    fn categories_by_game() {
        let l = names(&["iw4:cardicon_a", "iw4:cardicon_b", "jup:ui_emblem_005", "jup:ui_emblem_006"]);
        assert_eq!(categories_in(&l), vec![0, 2]);
        assert_eq!(in_category(&l, 1), names(&["jup:ui_emblem_005", "jup:ui_emblem_006"]));
        assert_eq!(in_category(&l, 7), names(&["jup:ui_emblem_005", "jup:ui_emblem_006"]));
        assert_eq!(category_of_in(&l, "jup:ui_emblem_006"), 1);
        assert_eq!(category_of_in(&l, "iw4:cardicon_b"), 0);
        assert_eq!(label("jup:ui_emblem_005"), "MWIII · EMBLEM 005");
    }

    #[test]
    fn places() {
        let l = names(&["a", "b", "c"]);
        assert_eq!(place_in(&l, "b"), (2, 3));
        assert_eq!(place_in(&l, "zz"), (0, 3));
    }
}
