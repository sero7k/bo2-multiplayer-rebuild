//! bo2mp: Combat Training's map rotation and the public lobby's map vote.
//! No game, no menus: the front end (`ui::lui_frontend`) feeds it the
//! maps and sends what it decides to the lobby.
//!
//! The maps offered are Custom Games' Change Map list (`Engine.GetMaps`:
//! every `mp_` row of BO2's mp/mapsTable.csv, in the table's order) less
//! any map that is not in the player's install.
//!
//! BO2's pick: a weighted draw over the playlist's maps, each map's weight
//! used up as it is drawn and every weight refilled once none is left (a
//! bag), never the map just played nor the one before it. A new public
//! party draws NEXT, then PREVIOUS (a second, different map). After a
//! match PREVIOUS is the map just played and NEXT a new draw. The weights
//! are in Treyarch's `playlists.info`, not on the player's disk. PLACEHOLDER: every
//! map weighs 1, so the bag gives each map once before any comes again.
//!
//! The vote is BO2's map voter's: NEXT, PREVIOUS and RANDOM, voted with
//! `xpartyveto 1` / `2` / `3`. Only the party's players vote (bots are not
//! in the party). The most votes win; ties go NEXT, then RANDOM, then
//! PREVIOUS; no votes keep NEXT. RANDOM is a new draw that is neither NEXT
//! nor PREVIOUS. PREVIOUS cannot win twice running
//! (`party_sequentialPreviousMapVotingAllowed`, off).

/// The maps a public lobby may offer: mapsTable's multiplayer rows (load
/// name in the first column) that `installed` has, in the table's order.
pub fn offered(maps_table: &[Vec<String>], installed: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in maps_table.iter().filter_map(|r| r.first()).map(|c| c.trim()) {
        if name.starts_with("mp_") && installed(name) && !out.iter().any(|m| m == name) {
            out.push(name.to_owned());
        }
    }
    out
}

/// A map's weight in the bag. PLACEHOLDER: `playlists.info`'s weights.
const WEIGHT: u32 = 1;

/// Combat Training's rotation: the map the next match plays (NEXT), the
/// one the lobby offers as PREVIOUS, and the bag they are drawn from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rotation {
    maps: Vec<String>,
    /// Each map's weight left in the bag.
    left: Vec<u32>,
    next: Option<String>,
    previous: Option<String>,
    /// The map the last match played (none before any).
    played: Option<String>,
    seed: u64,
}

impl Rotation {
    /// A new public party: NEXT, then PREVIOUS, two different draws
    /// (`seed` any number: the draws' dice).
    pub fn new(maps: Vec<String>, seed: u64) -> Self {
        let left = vec![WEIGHT; maps.len()];
        let mut r = Rotation { maps, left, next: None, previous: None, played: None, seed: seed | 1 };
        r.next = r.draw(&[]);
        let next = r.next.clone();
        r.previous = r.draw(&[next.as_deref().unwrap_or_default()]);
        r
    }

    pub fn maps(&self) -> &[String] {
        &self.maps
    }

    /// The map the next match plays (none when nothing is offered).
    pub fn next(&self) -> Option<&str> {
        self.next.as_deref()
    }

    /// The lobby's PREVIOUS: the map the last match played; before any,
    /// the party's second draw.
    pub fn previous(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    /// A match starts on `map`: it becomes PREVIOUS, and NEXT is a new draw
    /// that is neither `map` nor the map played before it (before any
    /// match, the lobby's other map).
    pub fn play(&mut self, map: &str) {
        let before = self.played.take().or_else(|| {
            if self.previous.as_deref() == Some(map) { self.next.clone() } else { self.previous.clone() }
        });
        self.played = Some(map.to_owned());
        self.previous = Some(map.to_owned());
        let skip: Vec<&str> = [Some(map), before.as_deref()].into_iter().flatten().collect();
        self.next = self.draw(&skip);
    }

    /// RANDOM's map: a new draw that is neither NEXT nor PREVIOUS.
    pub fn random(&mut self) -> Option<String> {
        let (next, previous) = (self.next.clone(), self.previous.clone());
        let skip: Vec<&str> = [next.as_deref(), previous.as_deref()].into_iter().flatten().collect();
        self.draw(&skip)
    }

    /// One draw from the bag, never a map in `skip` (unless every map is);
    /// the bag refills when no allowed map has weight left.
    fn draw(&mut self, skip: &[&str]) -> Option<String> {
        let allowed: Vec<usize> = (0..self.maps.len()).filter(|&i| !skip.contains(&self.maps[i].as_str())).collect();
        let allowed = if allowed.is_empty() { (0..self.maps.len()).collect() } else { allowed };
        if allowed.iter().all(|&i| self.left[i] == 0) {
            self.left.fill(WEIGHT);
        }
        let total: u64 = allowed.iter().map(|&i| u64::from(self.left[i])).sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.roll() % total;
        for &i in &allowed {
            let w = u64::from(self.left[i]);
            if roll < w {
                self.left[i] -= 1;
                return Some(self.maps[i].clone());
            }
            roll -= w;
        }
        None
    }

    /// The dice (xorshift).
    fn roll(&mut self) -> u64 {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.seed = x;
        x
    }
}

/// A map voter button, numbered as `xpartyveto` and the voter's
/// `selfVote` number them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Next = 1,
    Previous = 2,
    Random = 3,
}

impl Choice {
    pub fn from_number(n: i32) -> Option<Choice> {
        match n {
            1 => Some(Choice::Next),
            2 => Some(Choice::Previous),
            3 => Some(Choice::Random),
            _ => None,
        }
    }

    fn slot(self) -> usize {
        self as usize - 1
    }
}

/// The winner of a vote from its counts (NEXT, PREVIOUS, RANDOM): the
/// most votes; ties go NEXT, then RANDOM, then PREVIOUS.
pub fn winner(counts: [u32; 3]) -> Choice {
    let mut best = Choice::Next;
    for c in [Choice::Random, Choice::Previous] {
        if counts[c.slot()] > counts[best.slot()] {
            best = c;
        }
    }
    best
}

/// One public lobby's vote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vote {
    pub next: String,
    pub previous: String,
    /// PREVIOUS can be voted for: not when it won the lobby's last vote
    /// (BO2's `party_isPreviousMapVoted` greys it out).
    pub previous_allowed: bool,
    /// His vote.
    pub mine: Option<Choice>,
    /// The vote has closed on this map.
    pub chosen: Option<String>,
}

impl Vote {
    /// The vote on the rotation's next match (none when it offers no map).
    pub fn open(rotation: &Rotation, previous_allowed: bool) -> Option<Vote> {
        let next = rotation.next()?.to_owned();
        let previous = rotation.previous().unwrap_or(&next).to_owned();
        Some(Vote { next, previous, previous_allowed, mine: None, chosen: None })
    }

    /// He votes; false when the vote is closed or the choice is greyed out.
    pub fn cast(&mut self, c: Choice) -> bool {
        if self.chosen.is_some() || (c == Choice::Previous && !self.previous_allowed) {
            return false;
        }
        self.mine = Some(c);
        true
    }

    /// The votes for NEXT, PREVIOUS and RANDOM.
    pub fn counts(&self) -> [u32; 3] {
        let mut n = [0; 3];
        if let Some(c) = self.mine {
            n[c.slot()] += 1;
        }
        n
    }

    /// Close the vote: the winner and its map (RANDOM's drawn from
    /// `rotation`).
    pub fn close(&mut self, rotation: &mut Rotation) -> (Choice, String) {
        let c = winner(self.counts());
        let map = match c {
            Choice::Next => self.next.clone(),
            Choice::Previous => self.previous.clone(),
            Choice::Random => rotation.random().unwrap_or_else(|| self.next.clone()),
        };
        self.chosen = Some(map.clone());
        (c, map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(names: &[&str]) -> Vec<Vec<String>> {
        names.iter().map(|n| vec![n.to_string(), "x".into()]).collect()
    }

    fn rotation(names: &[&str], seed: u64) -> Rotation {
        Rotation::new(names.iter().map(|n| n.to_string()).collect(), seed)
    }

    const FIVE: [&str; 5] = ["mp_la", "mp_raid", "mp_slums", "mp_turbine", "mp_dig"];

    #[test]
    fn offers_installed_multiplayer_maps_in_table_order() {
        let t = table(&["name", "mp_la", "zm_transit", "mp_dig", "mp_nuketown_2020", " mp_raid "]);
        let installed = |m: &str| m != "mp_dig";
        assert_eq!(offered(&t, installed), ["mp_la", "mp_nuketown_2020", "mp_raid"]);
    }

    #[test]
    fn nothing_installed_offers_nothing() {
        let t = table(&["mp_la", "mp_raid"]);
        assert!(offered(&t, |_| false).is_empty());
        let mut r = Rotation::new(offered(&t, |_| false), 3);
        assert_eq!((r.next(), r.previous()), (None, None));
        assert_eq!(r.random(), None);
    }

    #[test]
    fn a_new_party_offers_two_different_maps() {
        for seed in 0..64 {
            let r = rotation(&FIVE, seed);
            let (n, p) = (r.next().unwrap(), r.previous().unwrap());
            assert_ne!(n, p);
            assert!(FIVE.contains(&n) && FIVE.contains(&p));
        }
        let firsts: std::collections::HashSet<String> = (0..64).map(|s| rotation(&FIVE, s).next().unwrap().to_owned()).collect();
        assert!(firsts.len() > 2, "{firsts:?}");
    }

    #[test]
    fn after_a_match_next_is_neither_of_the_last_two() {
        for seed in 0..32 {
            let mut r = rotation(&FIVE, seed);
            let mut before = r.previous().unwrap().to_owned();
            for _ in 0..20 {
                let m = r.next().unwrap().to_owned();
                r.play(&m);
                assert_eq!(r.previous(), Some(m.as_str()));
                let n = r.next().unwrap();
                assert!(n != m && n != before, "{n} after {m}, {before}");
                before = m;
            }
        }
    }

    #[test]
    fn the_bag_gives_every_map_before_any_comes_again() {
        for seed in 0..32 {
            let mut r = rotation(&FIVE, seed);
            let mut drawn = vec![r.previous().unwrap().to_owned(), r.next().unwrap().to_owned()];
            for _ in 0..3 {
                let m = r.next().unwrap().to_owned();
                r.play(&m);
                drawn.push(r.next().unwrap().to_owned());
            }
            drawn.sort();
            let mut all = FIVE.map(String::from).to_vec();
            all.sort();
            assert_eq!(drawn, all);
        }
    }

    #[test]
    fn one_map_is_every_choice() {
        let mut r = rotation(&["mp_la"], 5);
        assert_eq!((r.next(), r.previous()), (Some("mp_la"), Some("mp_la")));
        r.play("mp_la");
        assert_eq!((r.next(), r.previous()), (Some("mp_la"), Some("mp_la")));
    }

    #[test]
    fn no_votes_keep_the_next_map() {
        assert_eq!(winner([0, 0, 0]), Choice::Next);
        let mut r = rotation(&FIVE, 9);
        let mut v = Vote::open(&r, true).unwrap();
        let next = v.next.clone();
        assert_eq!(v.close(&mut r), (Choice::Next, next));
    }

    #[test]
    fn most_votes_win_and_ties_go_next_random_previous() {
        assert_eq!(winner([1, 2, 0]), Choice::Previous);
        assert_eq!(winner([0, 0, 1]), Choice::Random);
        assert_eq!(winner([2, 2, 2]), Choice::Next);
        assert_eq!(winner([0, 1, 1]), Choice::Random);
        assert_eq!(winner([1, 1, 0]), Choice::Next);
        assert_eq!(winner([1, 0, 1]), Choice::Next);
    }

    #[test]
    fn his_vote_moves_and_closes() {
        let mut r = rotation(&FIVE, 3);
        let mut v = Vote::open(&r, true).unwrap();
        assert_eq!((v.next.as_str(), v.previous.as_str()), (r.next().unwrap(), r.previous().unwrap()));
        assert!(v.cast(Choice::Random));
        assert!(v.cast(Choice::Previous));
        assert_eq!(v.counts(), [0, 1, 0]);
        let previous = v.previous.clone();
        assert_eq!(v.close(&mut r), (Choice::Previous, previous.clone()));
        assert!(!v.cast(Choice::Next));
        assert_eq!(v.chosen, Some(previous));
    }

    #[test]
    fn previous_greyed_out_cannot_be_voted() {
        let mut r = rotation(&FIVE, 1);
        let mut v = Vote::open(&r, false).unwrap();
        assert!(!v.cast(Choice::Previous));
        assert_eq!(v.counts(), [0, 0, 0]);
        assert_eq!(v.close(&mut r).0, Choice::Next);
    }

    #[test]
    fn random_is_neither_next_nor_previous() {
        for seed in 0..32 {
            let mut r = rotation(&FIVE, seed);
            let mut v = Vote::open(&r, true).unwrap();
            assert!(v.cast(Choice::Random));
            let (c, m) = v.close(&mut r);
            assert_eq!(c, Choice::Random);
            assert!(m != v.next && m != v.previous, "{m}");
        }
        // (A map not installed is not in the rotation, so never RANDOM's.)
        let t = table(&["mp_la", "mp_dig", "mp_raid", "mp_slums"]);
        for seed in 0..16 {
            let mut r = Rotation::new(offered(&t, |m| m != "mp_dig"), seed);
            assert_ne!(r.random().as_deref(), Some("mp_dig"));
        }
    }
}
