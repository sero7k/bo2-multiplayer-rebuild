//! bo2mp shell: WARZONE's look and words, kept in one place so they can be
//! restyled without touching the screens (`wz_view`) or what the rows do
//! (`warzone`).
//!
//! The WARZONE screens are Multiplayer's screens again (its menu, a private
//! match lobby, a mode picker, and Create-a-Class, Barracks and Options
//! opened from them) drawn in `PALETTE` instead of Multiplayer's blue.

use bevy::prelude::*;

use super::theme::Palette;

/// WARZONE's colours: dark military greys with a hazard yellow-orange.
/// (Each field is what `Palette` says it is; `accent` is the bar by the
/// header, the row in focus, tabs, selector arrows and panel edges.)
pub(crate) const PALETTE: Palette = Palette {
    accent: Color::srgb(0.98, 0.66, 0.10),
    accent_soft: Color::srgba(0.98, 0.66, 0.10, 0.30),
    panel: Color::srgba(0.055, 0.058, 0.052, 0.86),
    panel_soft: Color::srgba(0.055, 0.058, 0.052, 0.58),
    line: Color::srgba(0.92, 0.86, 0.70, 0.14),
    footer: Color::srgba(0.030, 0.032, 0.028, 0.76),
    strip: Color::srgba(0.030, 0.030, 0.024, 0.88),
    picture_bg: Color::srgb(0.09, 0.09, 0.08),
    shade: [0.018, 0.019, 0.016],
    tint: Color::srgba(0.16, 0.11, 0.02, 0.22),
};

/// The other squads' colour in the lobby's squad list.
pub(crate) const OTHER_SQUAD: Color = Color::srgb(0.52, 0.54, 0.50);

/// The first word of every WARZONE screen's header.
pub(crate) const HEADER: &str = "WARZONE";

/// WARZONE's menu: (label, line under it in focus).
pub(crate) const QUICK_PLAY: (&str, &str) = ("QUICK PLAY", "Pick a playlist and deploy at once with bot squads.");
pub(crate) const PRIVATE_MATCH: (&str, &str) = ("PRIVATE MATCH", "Your own lobby: the mode, squad size, squads and bots, then deploy.");
pub(crate) const LOADOUTS: (&str, &str) = ("CREATE A CLASS", "Build the loadouts your loadout drops bring.");
pub(crate) const BARRACKS: (&str, &str) = ("BARRACKS", "Your combat record, calling card, emblem and prestige.");
pub(crate) const OPTIONS: (&str, &str) = ("OPTIONS", "Graphics, sound and controls.");

/// The private match lobby: its header and its tabs (lobby, loadouts,
/// settings).
pub(crate) const LOBBY_HEADER: [&str; 2] = ["PRIVATE MATCH", "SQUAD LOBBY"];
pub(crate) const TABS: [&str; 3] = ["SQUAD", "LOADOUTS", "SETTINGS"];
pub(crate) const DEPLOY: (&str, &str) = ("DEPLOY", "Everyone drops in. Last squad standing wins.");
pub(crate) const CHANGE_MODE: (&str, &str) = ("CHANGE MODE", "Resurgence on Rebirth Island, Solos to Quads.");
pub(crate) const GAME_MODE: &str = "GAME MODE";
pub(crate) const SQUAD_SIZE: &str = "SQUAD SIZE";
pub(crate) const SQUADS: (&str, &str) = ("SQUADS", "Your squad and the bot squads, filled with bots.");
pub(crate) const BOT_DIFFICULTY: &str = "BOT DIFFICULTY";
pub(crate) const MAP: &str = "MAP";
/// The lobby's status line: ready, or deploying in N.
pub(crate) const READY: &str = "Ready to deploy";
pub(crate) const DEPLOYING_IN: &str = "Deploying in";
pub(crate) const MY_SQUAD: &str = "YOUR SQUAD";
pub(crate) const SQUAD: &str = "SQUAD";

/// The mode picker's header, and the word over its list.
pub(crate) const MODE_HEADER: &str = "SELECT MODE";
pub(crate) const FEATURED: &str = "FEATURED PLAYLIST";
/// The squad list's heading for the bot squads.
pub(crate) const ENEMY_SQUADS: &str = "ENEMY SQUADS";
/// The footer's way out of the lobby.
pub(crate) const LEAVE_LOBBY: &str = "LEAVE LOBBY";
