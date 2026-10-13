//! bo2mp: the lobby's bots. The pregame lobby lists each bot by the name and
//! level it will play the match under: the bots that join take these names
//! in the lobby's order (friends' places first, then enemies'), and each
//! bot's level comes from its name, so the lobby, the scoreboard and the
//! killfeed agree.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The made-up gamertags (some with a clan tag), one per line.
const NAMES: &str = include_str!("../../bots/src/bot_names.txt");

fn names() -> &'static [&'static str] {
    static LIST: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    LIST.get_or_init(|| NAMES.lines().map(str::trim).filter(|l| !l.is_empty()).collect())
}

/// Where this run's walk through the names starts (random per run).
fn start() -> usize {
    static START: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *START.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos() as usize ^ d.as_secs() as usize)
    })
}

/// The lobby's current draw (moved on after each match, so the next lobby
/// shows new faces).
static ROUND: AtomicUsize = AtomicUsize::new(0);
/// How many of the lobby's names the joining bots have taken.
static HANDED: AtomicUsize = AtomicUsize::new(0);
static ROUND_LOCK: Mutex<()> = Mutex::new(());

const LOBBY: usize = 18;

/// The `slot`th bot of the lobby (1 first): its name.
#[must_use]
pub fn lobby_bot_name(slot: u32) -> String {
    let n = names();
    if n.is_empty() {
        return format!("Bot {slot}");
    }
    let round = ROUND.load(Ordering::Relaxed);
    let i = round * LOBBY + slot.saturating_sub(1) as usize;
    // 7919 is prime: every index once before any comes back.
    n[(start() + i * 7919) % n.len()].to_owned()
}

/// A bot's level from its name (4 to 55, the same everywhere it shows).
#[must_use]
pub fn level_for(name: &str) -> i32 {
    // (The match keeps a name's first 15 bytes: hash what both sides see.)
    let mut h: u32 = 2166136261;
    for b in name.bytes().take(15) {
        h = (h ^ u32::from(b)).wrapping_mul(16777619);
    }
    4 + (h % 52) as i32
}

/// The next joining bot's name: the lobby's, in its order, then fresh ones.
#[must_use]
pub fn next_name() -> String {
    let _g = ROUND_LOCK.lock();
    let k = HANDED.fetch_add(1, Ordering::Relaxed);
    lobby_bot_name(k as u32 + 1)
}

/// A match ended: the next lobby draws new names.
pub fn next_lobby() {
    let _g = ROUND_LOCK.lock();
    if HANDED.swap(0, Ordering::Relaxed) > 0 {
        ROUND.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn joining_bots_take_the_lobby_names_in_order() {
        let a = super::lobby_bot_name(1);
        let b = super::lobby_bot_name(2);
        assert_ne!(a, b);
        assert_eq!(super::next_name(), a);
        assert_eq!(super::next_name(), b);
        let l = super::level_for(&a);
        assert!((4..=55).contains(&l));
        assert_eq!(l, super::level_for(&a));
        super::next_lobby();
    }
}
