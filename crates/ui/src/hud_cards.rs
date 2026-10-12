//! bo2mp: the players' calling cards and emblems in the match HUD (the kill
//! card, the Killed By card, the killcam's name card). Black Ops II's
//! banners and emblems are not shown: each player has a Modern Warfare 2
//! or 3 calling card (`cardtitle_*`) and emblem (`cardicon_*`), read from
//! that game's own files (`iw4:<material>` / `iw5:<material>`). His own are
//! the ones his stats hold (`bo2mp.callingcard` / `bo2mp.emblem`, picked in
//! the shell's Barracks); anyone else's follow from the name (the shell's
//! `cards::for_name`), so a player keeps his for good. A picture the games
//! do not have leaves the plain drawn card (never Black Ops II's).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::shell::cards::{self, Kind};

/// His calling card and emblem as his stats hold them (empty: none).
pub(crate) fn his_look_stats(profile: &bo2_profile::Profile) -> (String, String) {
    let read = |key: &str| match profile.stat(key) {
        Some(bo2_profile::StatValue::Str(t)) => t.trim().to_owned(),
        _ => String::new(),
    };
    (read(cards::CARD_STAT), read(cards::EMBLEM_STAT))
}

/// A player's calling card and emblem (`iw4:cardtitle_*`, `iw4:cardicon_*`):
/// `mine` holds his own stats' values when the player is him.
pub(crate) fn look_of(name: &str, mine: Option<&(String, String)>) -> (String, String) {
    let look = match mine {
        Some((card, emblem)) => (cards::mine(Kind::Card, card), cards::mine(Kind::Emblem, emblem)),
        None => (cards::for_name(Kind::Card, name), cards::for_name(Kind::Emblem, name)),
    };
    // (Once per player and look: which materials he got.)
    static SAID: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let line = format!("{name} -> card {} emblem {}{}", look.0, look.1, if mine.is_some() { " (his own)" } else { "" });
    if let Ok(mut said) = SAID.lock()
        && said.get_or_insert_with(HashSet::new).insert(line.clone())
    {
        diag::info!(Ui, "bo2mp hud: look: {line}");
    }
    look
}

/// A calling card's or emblem's picture (`iw4:` / `iw5:` material), read
/// off the main thread from its game: `None` while it is read, and for good
/// when the game lacks it (the reader logs which).
pub(crate) fn picture(material: &str) -> Option<Arc<Image>> {
    let (game, name) = material.split_once(':')?;
    if !matches!(game, "iw4" | "iw5") || !(name.starts_with("cardtitle_") || name.starts_with("cardicon_")) {
        return None;
    }
    let (game, name) = (game.to_owned(), name.to_owned());
    let img = crate::foreign_maps::picture_async(material, true, move || crate::foreign_maps::read_game_image(&game, &[name]))?;
    // (Once per material: the first time it is in.)
    static READ: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    if let Ok(mut read) = READ.lock()
        && read.get_or_insert_with(HashSet::new).insert(material.to_owned())
    {
        diag::info!(Ui, "bo2mp hud: {material} read ({}x{})", img.width(), img.height());
    }
    Some(img)
}

/// Whether a material is one of these (a calling card or an emblem).
pub(crate) fn is_look(material: &str) -> bool {
    material.split_once(':').is_some_and(|(g, n)| matches!(g, "iw4" | "iw5") && (n.starts_with("cardtitle_") || n.starts_with("cardicon_")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_are_steady_and_from_the_games() {
        let a = look_of("WildBacon13", None);
        assert_eq!(a, look_of("WildBacon13", None));
        assert!(a.0.contains(":cardtitle_") && a.1.contains(":cardicon_"));
        assert!(is_look(&a.0) && is_look(&a.1));
        assert!(!is_look("emblem_bg_default") && !is_look("em_rank_pvt_full"));
        // His own: what his stats hold when the games have it.
        let list = cards::list(Kind::Card);
        let mine = look_of("sero", Some(&(list[list.len() - 1].clone(), String::new())));
        assert_eq!(mine.0, list[list.len() - 1]);
    }
}
