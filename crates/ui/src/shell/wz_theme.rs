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

/// The Warzone HUD's colours (`hud_warzone`: Resurgence in a match). Its
/// pictures are Modern Warfare III's own, white glyphs drawn in these.
pub(crate) struct WzHud {
    /// Headings, the circle panel's edge, the contract, the focus.
    pub accent: Color,
    /// Values (cash, counts, names, the clip).
    pub text: Color,
    /// Labels and secondary values.
    pub dim: Color,
    /// Panels behind the circle, contract, counters and squad rows.
    pub panel: Color,
    /// Empty bar tracks.
    pub track: Color,
    /// Armor: the plate segments and the plate icons (Modern Warfare III's
    /// blue).
    pub armor: Color,
    pub health: Color,
    /// Health at a third or less, the gas warning's skull.
    pub low: Color,
    pub cash: Color,
    /// The gas: its warning and its time while the circle closes.
    pub gas: Color,
    /// The safe zone's ring on the minimap, and the next one's.
    pub circle: Color,
    pub next_circle: Color,
    /// A squadmate down (dead, waiting) and one redeploying.
    pub down: Color,
    pub redeploy: Color,
    /// Enemies: kill splashes, danger pings, the kill hit marker.
    pub enemy: Color,
    /// Hit markers on armor, and a plate breaking.
    pub armor_hit: Color,
    pub armor_break: Color,
    /// Each squad slot's colour (him first), as Warzone numbers them.
    pub squad: [Color; 4],
}

pub(crate) const HUD: WzHud = WzHud {
    accent: Color::srgb(0.98, 0.66, 0.10),
    text: Color::srgb(0.96, 0.94, 0.88),
    dim: Color::srgba(0.92, 0.86, 0.70, 0.66),
    panel: Color::srgba(0.035, 0.037, 0.032, 0.70),
    track: Color::srgba(0.0, 0.0, 0.0, 0.50),
    armor: Color::srgb(0.98, 0.74, 0.26),
    health: Color::srgb(0.95, 0.94, 0.90),
    low: Color::srgb(0.92, 0.28, 0.18),
    cash: Color::srgb(0.66, 0.88, 0.42),
    gas: Color::srgb(0.96, 0.40, 0.14),
    circle: Color::srgba(0.98, 0.66, 0.10, 0.95),
    next_circle: Color::srgba(0.96, 0.94, 0.88, 0.70),
    down: Color::srgb(0.92, 0.28, 0.18),
    redeploy: Color::srgb(0.98, 0.80, 0.40),
    enemy: Color::srgb(0.92, 0.26, 0.20),
    armor_hit: Color::srgb(0.98, 0.74, 0.26),
    armor_break: Color::srgb(1.0, 0.86, 0.45),
    squad: [
        Color::srgb(0.98, 0.66, 0.10),
        Color::srgb(0.40, 0.76, 0.82),
        Color::srgb(0.58, 0.82, 0.38),
        Color::srgb(0.88, 0.50, 0.72),
    ],
};

/// The Warzone HUD's words.
pub(crate) const HUD_WORDS: HudWords = HudWords {
    infil: "INFIL",
    circle_closes: "CIRCLE CLOSES",
    circle_closing: "CIRCLE CLOSING",
    final_circle: "FINAL CIRCLE",
    in_gas: "YOU ARE IN THE GAS",
    gas_mask: "GAS MASK",
    redeploying: "REDEPLOYING IN",
    redeploy_off: "REDEPLOY DISABLED",
    buy_back: "A squadmate can buy you back",
    squads_left: "SQUADS",
    players_left: "PLAYERS",
    kills: "KILLS",
    plates: "PLATES",
    eliminated: "ELIMINATED",
    victory: "VICTORY",
    squad_placed: "SQUAD PLACED",
    of: "OF",
};

pub(crate) struct HudWords {
    pub infil: &'static str,
    pub circle_closes: &'static str,
    pub circle_closing: &'static str,
    pub final_circle: &'static str,
    pub in_gas: &'static str,
    pub gas_mask: &'static str,
    pub redeploying: &'static str,
    pub redeploy_off: &'static str,
    pub buy_back: &'static str,
    pub squads_left: &'static str,
    pub players_left: &'static str,
    pub kills: &'static str,
    pub plates: &'static str,
    pub eliminated: &'static str,
    /// The end screen (`hud_outcome`): his squad won; its place.
    pub victory: &'static str,
    pub squad_placed: &'static str,
    pub of: &'static str,
}

/// The other squads' colour in the lobby's squad list.
pub(crate) const OTHER_SQUAD: Color = Color::srgb(0.52, 0.54, 0.50);

/// The first word of every WARZONE screen's header.
pub(crate) const HEADER: &str = "DROP ZONE";

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
