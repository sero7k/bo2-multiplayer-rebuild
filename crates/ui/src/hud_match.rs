//! bo2mp: the match's own kill feed, hit marker, "YOU KILLED" banner and
//! scoreboard, in the shell's look (navy panels, the blue accent, Barlow
//! Condensed), drawn in place of BO2's LUI versions while the modern HUD is
//! on (`hud_modern::modern_hud_enabled`):
//!
//! - Kill feed (bottom left, above his name): "Attacker [kill icon] Victim"
//!   rows from the server's `bo2mp_feed`, names in team colours (his side
//!   blue, the enemy red, him gold), a headshot icon on a headshot, at most
//!   five rows, each fading after six seconds. BO2's obituary window is left
//!   empty (`lui_frontend::match_events`).
//! - Hit marker: a white X on each hit (BO2's `damage_feedback` script
//!   element in `bo2zm_hud` coming back up), red on the killing shot (his
//!   kill in the feed), a quick scale pop and a ~0.3 s fade. BO2's picture
//!   is left out of the script HUD (`zm_hud`).
//! - "YOU KILLED": the victim's card under the crosshair: an emblem, his
//!   name, his title and his rank icon and level, on a Modern Warfare 2
//!   calling card (`cardtitle_*`, emblems `cardicon_*`, read from MW2's
//!   IWDs; title and card picked by the player's name, as no one has chosen
//!   one yet). Not for the match's last kill (the final killcam has it).
//! - Scoreboard: while his scores key is held and at the match's end (BO2's
//!   LUI `Scoreboard` menu is not opened then, `lui_hud`): the game type and
//!   the clock, then each side's players (or one list in a free-for-all)
//!   with rank, score, kills, assists, deaths, K/D and ping.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::{GameUiFont, UiLayer, UiLayerVisibility};
use crate::shell::theme::{ACCENT, DIM, ENEMY, FAINT, GOLD, LINE, PANEL, ShellFonts, TEXT, WHITE};

/// Free-for-all game types (one list, no sides).
const FFA_TYPES: [&str; 5] = ["dm", "gun", "oic", "sas", "shrp"];

/// The kill feed: rows shown at once, how long each stays (s), its fades.
const FEED_ROWS: usize = 5;
const FEED_LIFE: f32 = 6.0;
const FEED_IN: f32 = 0.18;
const FEED_OUT: f32 = 0.6;

/// The hit marker's life (s): a hit, the killing shot.
const HIT_LIFE: f32 = 0.30;
const KILL_LIFE: f32 = 0.42;
/// The marker's pop: it starts this much bigger and settles in this long.
const POP: f32 = 0.38;
const POP_TIME: f32 = 0.07;
/// A kill in the feed this soon after a hit turns that hit's marker red.
const KILL_AFTER_HIT: f64 = 0.35;

/// The kill banner's fades and hold (s), and how long a kill waits before
/// its banner (the match may have ended on it).
const BANNER_IN: f32 = 0.14;
const BANNER_HOLD: f32 = 2.3;
const BANNER_OUT: f32 = 0.45;
const BANNER_WAIT: f64 = 0.22;

const HIT_RED: Color = Color::srgb(0.96, 0.16, 0.12);
const MINE_ROW: Color = Color::srgba(0.26, 0.62, 0.95, 0.24);
const ENEMY_ROW: Color = Color::srgba(0.90, 0.26, 0.22, 0.24);
const PLAIN_ROW: Color = Color::srgba(0.70, 0.80, 0.95, 0.06);

// --- Shared: faces, fades, pictures ---------------------------------------

/// The two weights the HUD writes in.
#[derive(Clone)]
pub(crate) struct Faces {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
}

impl Faces {
    pub(crate) fn of(shell: Option<&ShellFonts>, game: Option<&GameUiFont>) -> Option<Faces> {
        match (shell, game) {
            (Some(s), _) => Some(Faces { regular: s.regular.clone(), bold: s.bold.clone() }),
            (None, Some(g)) => Some(Faces { regular: g.0.clone(), bold: g.0.clone() }),
            _ => None,
        }
    }
}

fn font(h: &Handle<Font>, px: f32) -> TextFont {
    crate::shell::theme::font(h, px)
}

fn shadow(px: f32) -> TextShadow {
    TextShadow { offset: Vec2::splat((px / 14.0).max(1.0)), color: Color::srgba(0.0, 0.0, 0.0, 0.75), ..default() }
}

/// A piece that fades in, holds and fades out (seconds from `start`, on
/// the real clock); `alpha` is its own colour's alpha.
#[derive(Component, Clone, Copy, Debug)]
struct Fade {
    start: f64,
    fade_in: f32,
    life: f32,
    out: f32,
    alpha: f32,
    shadow: f32,
}

impl Fade {
    fn new(start: f64, fade_in: f32, life: f32, out: f32) -> Self {
        Fade { start, fade_in, life, out, alpha: 1.0, shadow: 0.75 }
    }

    fn with(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    fn at(&self, now: f64) -> f32 {
        let t = (now - self.start) as f32;
        let fin = if self.fade_in > 0.0 { (t / self.fade_in).clamp(0.0, 1.0) } else { 1.0 };
        let fout = if self.out > 0.0 { ((self.life - t) / self.out).clamp(0.0, 1.0) } else { f32::from(t < self.life) };
        fin.min(fout)
    }
}

/// A `Fade` that fades the node's background too (a panel).
#[derive(Component, Clone, Copy, Debug)]
struct FadeBack;

fn now_secs(time: &Time<Real>) -> f64 {
    time.elapsed_secs_f64()
}

/// Each fading piece's colour for now.
#[allow(clippy::type_complexity)]
fn apply_fades(
    time: Res<Time<Real>>,
    mut q: Query<(
        &Fade,
        Option<&mut TextColor>,
        Option<&mut ImageNode>,
        Option<&mut BackgroundColor>,
        Option<&mut TextShadow>,
        Has<FadeBack>,
    )>,
) {
    let now = now_secs(&time);
    for (f, text, image, back, shade, fades_back) in &mut q {
        let a = f.at(now);
        if let Some(mut c) = text {
            let want = c.0.with_alpha(f.alpha * a);
            if c.0 != want {
                c.0 = want;
            }
        }
        if let Some(mut i) = image {
            let want = i.color.with_alpha(f.alpha * a);
            if i.color != want {
                i.color = want;
            }
        }
        // (Only a panel's own background: every UI node has one, clear.)
        if let Some(mut b) = back.filter(|_| fades_back) {
            let want = b.0.with_alpha(f.alpha * a);
            if b.0 != want {
                b.0 = want;
            }
        }
        if let Some(mut s) = shade {
            let want = s.color.with_alpha(f.shadow * a);
            if s.color != want {
                s.color = want;
            }
        }
    }
}

/// A picture from BO2's HUD pictures by its exact name, made once.
fn hud_picture(
    cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
    name: &str,
) -> Option<(Handle<Image>, Vec2)> {
    if let Some(found) = cache.get(name) {
        return found.clone();
    }
    let icons = icons?;
    let found = icons.0.iter().find(|(k, _)| k == name).map(|(_, img)| {
        let size = img.size().as_vec2();
        (images.add((**img).clone()), size)
    });
    cache.insert(name.to_owned(), found.clone());
    found
}

/// A picture read from another game's files (off the main thread: `None`
/// until it is in), made once.
fn game_picture(
    cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>,
    images: &mut Assets<Image>,
    game: &'static str,
    name: &str,
) -> Option<(Handle<Image>, Vec2)> {
    let key = format!("{game}:{name}");
    if let Some(Some(found)) = cache.get(&key) {
        return Some(found.clone());
    }
    let owned = name.to_owned();
    let img = crate::foreign_maps::picture_async(&key, true, move || {
        crate::foreign_maps::read_game_image(game, std::slice::from_ref(&owned))
    })?;
    let size = img.size().as_vec2();
    let mut img = (*img).clone();
    img.sampler = bevy::image::ImageSampler::linear();
    let h = images.add(img);
    cache.insert(key, Some((h.clone(), size)));
    Some((h, size))
}

/// The weapon's kill icon (`killicon:<weapon>:<ratio>`, BO2's WeaponDef
/// killIcon) or, with no gun, the picture of how he died, and its width per
/// height: the same rule as BO2's obituary (`lui_frontend::feed_line`).
fn kill_icon_name(icons: &assets::T6HudIcons, weapon: &str, means: &str) -> Option<(String, f32)> {
    let base = weapon.split('+').next().unwrap_or("").trim_end_matches("_mp");
    let named = |n: &str| -> Option<(String, f32)> {
        let prefix = format!("killicon:{n}:");
        icons.0.iter().find_map(|(k, _)| {
            let ratio = k.strip_prefix(&prefix)?;
            Some((
                k.clone(),
                match ratio {
                    "1" => 2.0,
                    "2" => 4.0,
                    _ => 1.0,
                },
            ))
        })
    };
    if base.is_empty() || base == "none" {
        match means {
            "MOD_MELEE" => named("hud_obit_knife"),
            "MOD_HEAD_SHOT" => named("killiconheadshot"),
            "MOD_CRUSH" => named("hud_obit_death_crush"),
            "MOD_FALLING" => named("hud_obit_death_falling"),
            "MOD_SUICIDE" | "MOD_TRIGGER_HURT" => named("hud_obit_death_suicide"),
            "MOD_IMPACT" => named("hud_obit_death_grenade_round"),
            _ => None,
        }
        .or_else(|| named("killicondied"))
    } else {
        named(base).or_else(|| named("hud_obit_death_suicide"))
    }
}

/// A stable number for a name (FNV-1a): the same player gets the same
/// title, card and emblem every match.
fn name_hash(name: &str, salt: u32) -> u32 {
    let mut h: u32 = 0x811c_9dc5 ^ salt.wrapping_mul(0x9e37_79b9);
    for b in name.trim().to_ascii_lowercase().bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h ^ (h >> 15)
}

// --- What the server sent ---------------------------------------------------

/// Whose side a name is on, as he sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    You,
    Mine,
    Enemy,
    Neutral,
}

impl Side {
    fn name_color(self) -> Color {
        match self {
            Side::You => GOLD,
            Side::Mine => ACCENT,
            Side::Enemy => ENEMY,
            Side::Neutral => WHITE,
        }
    }
}

/// One kill as `bo2mp_feed` carries it
/// ("<n>|<attacker>|<victim>|<weapon>|<mod>|<self>|<attacker team>|<victim team>").
#[derive(Clone, Debug, PartialEq)]
struct Kill {
    n: u32,
    attacker: String,
    victim: String,
    weapon: String,
    means: String,
    suicide: bool,
    attacker_team: String,
    victim_team: String,
}

fn parse_feed(feed: &str) -> Vec<Kill> {
    feed.split(';')
        .filter_map(|item| {
            let f: Vec<&str> = item.split('|').collect();
            let n = f.first()?.parse::<u32>().ok()?;
            let get = |i: usize| f.get(i).copied().unwrap_or("").to_owned();
            Some(Kill {
                n,
                attacker: get(1),
                victim: get(2),
                weapon: get(3),
                means: get(4),
                suicide: f.get(5) == Some(&"self"),
                attacker_team: get(6),
                victim_team: get(7),
            })
        })
        .collect()
}

/// The match as his screen needs it this frame.
pub(crate) struct Seen {
    me: String,
    my_team: String,
    ffa: bool,
    pub(crate) gametype: String,
    kills: Vec<Kill>,
    /// The HUD is up (no killcam, no menu, not taken away).
    pub(crate) hud: bool,
    /// The match is over or ending on this frame.
    pub(crate) ended: bool,
    timeleft: Option<i32>,
    scores: (i32, i32),
    limit: i32,
    teams: HashMap<u32, String>,
    /// BO2's hit marker element's alpha (0: none).
    marker_alpha: f32,
}

pub(crate) fn read_seen(presented: &PresentedSnapshot, local: &LocalPresentClient, menus: Option<&frame::LuiMenus>) -> Option<Seen> {
    let snap = presented.snapshot()?;
    let dvars = snap.meta.script_dvars(local.0);
    let s = |k: &str| dvars.string(k).unwrap_or_default().to_owned();
    let gametype = dvars.string("bo2mp_gametype")?.to_owned();
    let meta = snap.meta.for_client(local.0)?;
    let me = String::from_utf8_lossy(&meta.name).trim_end_matches(char::from(0)).to_owned();
    let my_team = s("bo2mp_team");
    let ffa = FFA_TYPES.contains(&gametype.as_str()) || my_team == "free";
    let killcam = s("bo2mp_killcam");
    let hud_hidden = dvars.string("bo2zm_hud_hidden") == Some("1");
    let menu_open = menus.is_some_and(|m| m.open) || s("bo2mp_menu").split_once(':').is_some_and(|(_, m)| !m.trim().is_empty());
    let in_killcam = killcam.starts_with("1|") || killcam.starts_with("2|");
    let settings = s("bo2mp_settings");
    let limit = settings
        .split(',')
        .find_map(|kv| kv.split_once('=').filter(|(k, _)| k.trim().eq_ignore_ascii_case("scorelimit")))
        .and_then(|(_, v)| v.trim().parse::<f32>().ok())
        .map_or(0, |v| v as i32);
    let scores = s("bo2mp_scores");
    let (a, b) = scores.split_once(',').unwrap_or(("0", "0"));
    let num = |x: &str| x.trim().parse::<i32>().unwrap_or(0);
    let teams = s("bo2mp_teams")
        .split(',')
        .filter_map(|e| {
            let (c, t) = e.split_once(':')?;
            Some((c.trim().parse::<u32>().ok()?, t.trim().to_owned()))
        })
        .collect();
    let marker_alpha = dvars
        .string("bo2zm_hud")
        .unwrap_or_default()
        .split('\u{1e}')
        .filter(|row| row.contains("#shader:damage_feedback"))
        .filter_map(|row| row.split('\u{1f}').nth(10)?.parse::<f32>().ok())
        .fold(0.0f32, f32::max);
    Some(Seen {
        me,
        my_team,
        ffa,
        gametype,
        kills: parse_feed(&s("bo2mp_feed")),
        hud: !hud_hidden && !in_killcam && !menu_open,
        ended: dvars.string("bo2mp_game_ended") == Some("1") || killcam.starts_with("2|") || hud_hidden,
        timeleft: s("bo2mp_timeleft").trim().parse::<f32>().ok().map(|t| t.max(0.0) as i32),
        scores: (num(a), num(b)),
        limit,
        teams,
        marker_alpha,
    })
}

impl Seen {
    fn side_of(&self, name: &str, team: &str) -> Side {
        if !name.is_empty() && name == self.me {
            return Side::You;
        }
        let sided = |t: &str| matches!(t, "allies" | "axis");
        if self.ffa || !sided(team) || !sided(&self.my_team) {
            Side::Neutral
        } else if team == self.my_team {
            Side::Mine
        } else {
            Side::Enemy
        }
    }

    /// His kills (not a suicide, not himself).
    fn is_my_kill(&self, k: &Kill) -> bool {
        !k.suicide && !self.me.is_empty() && k.attacker == self.me && k.victim != self.me
    }

    /// The match ended (or ends on this kill): his side's or his own score
    /// reached the limit.
    fn at_limit(&self, my_score: i32) -> bool {
        if self.limit <= 0 {
            return false;
        }
        let mine = if self.ffa {
            my_score
        } else if self.my_team == "axis" {
            self.scores.1
        } else {
            self.scores.0
        };
        mine >= self.limit
    }
}

/// The new kills since `last` (the server's count starts again in a new
/// match: a feed all below `last` resets it).
fn new_kills<'a>(kills: &'a [Kill], last: &mut Option<u32>) -> Vec<&'a Kill> {
    let top = kills.iter().map(|k| k.n).max();
    let Some(seen) = *last else {
        // (The first look: what is there is old.)
        *last = Some(top.unwrap_or(0));
        return Vec::new();
    };
    if top.is_some_and(|t| t < seen) {
        *last = Some(0);
    }
    let from = last.unwrap_or(0);
    let fresh: Vec<&Kill> = kills.iter().filter(|k| k.n > from).collect();
    if let Some(t) = top {
        *last = Some(t.max(from));
    }
    fresh
}

// --- Modern Warfare 2's cards, titles and Modern Warfare 3's rank icons ----

/// What another game's files give the banner and the rank icons, found
/// once off the main thread (each list empty when its game is not there).
#[derive(Clone, Debug, Default)]
struct Catalog {
    /// MW2's calling card banners (`cardtitle_*` images).
    cards: Vec<String>,
    /// MW2's emblems (`cardicon_*` images).
    emblems: Vec<String>,
    /// MW2's titles (its own text where found, else `MW2_TITLES`).
    titles: Vec<String>,
    /// MW3's level icons in rank order (`rank_*` images): the game and its
    /// pictures.
    ranks: Option<(&'static str, Vec<String>)>,
}

/// Modern Warfare 2 titles, used when its text could not be read.
const MW2_TITLES: &[&str] = &[
    "Gun Runner", "Bulletproof", "Ghost", "Tactician", "Brutal", "No Mercy", "The Bomb", "Ninja",
    "Kingslayer", "Truly Elite", "Overkill", "Blindfire", "Lifer", "Collateral", "Hard Landing",
    "Hot Potato", "Survivalist", "Mach 5", "Grim Reaper", "Fearless", "Flawless", "Rapid Fire",
    "Juggernaut", "Sharpshooter", "Afterburner", "Carpenter", "Hijacker", "Unbelievable",
];

/// MW2/MW3 rank stems in order (some ranks have two spellings).
const RANK_ORDER: &[&[&str]] = &[
    &["pvt"],
    &["pfc"],
    &["spc", "spec"],
    &["cpl"],
    &["sgt"],
    &["ssgt"],
    &["sfc"],
    &["msgt"],
    &["1stsgt", "fsgt", "1sg"],
    &["sgtmaj", "sgm"],
    &["cmdsgtmaj", "csm"],
    &["2ndlt", "2lt"],
    &["1stlt", "1lt"],
    &["capt", "cpt"],
    &["maj"],
    &["ltcol"],
    &["col"],
    &["bgen", "brgen", "brig"],
    &["majgen"],
    &["ltgen"],
    &["gen"],
    &["comm"],
];

/// A `rank_<stem><n>` name's place in the ladder: (stem order, n).
fn rank_place(name: &str) -> Option<(usize, u32)> {
    let rest = name.strip_prefix("rank_")?;
    if rest.starts_with("prestige") {
        return None;
    }
    // (Whole stems: `sgt` must not take `sgtmaj1`, and `1stsgt` starts
    // with a digit.)
    RANK_ORDER.iter().enumerate().find_map(|(i, stems)| {
        stems.iter().find_map(|s| {
            let tail = rest.strip_prefix(s)?;
            if tail.is_empty() {
                return Some((i, 0));
            }
            tail.parse::<u32>().ok().map(|n| (i, n))
        })
    })
}

/// MW3's levels (its level icons spread over these).
const MW3_LEVELS: i32 = 80;

fn catalog() -> Option<Catalog> {
    static FOUND: OnceLock<Mutex<Option<Catalog>>> = OnceLock::new();
    let slot = FOUND.get_or_init(|| {
        std::thread::Builder::new()
            .name("hud-cards".into())
            .spawn(|| {
                let found = find_catalog();
                if let Some(s) = FOUND.get()
                    && let Ok(mut g) = s.lock()
                {
                    *g = Some(found);
                }
            })
            .ok();
        Mutex::new(None)
    });
    slot.lock().ok()?.clone()
}

fn image_names(game: asset_core::AssetNamespace) -> Vec<String> {
    let Ok(root) = asset_transport::games_root_from_env() else { return Vec::new() };
    let trees = asset_transport::NamespaceTrees::discover(&root);
    let Some(main) = trees.main_for(game) else { return Vec::new() };
    match asset_transport::IwdIndex::open(main) {
        Ok(index) => index.image_names().map(str::to_owned).collect(),
        Err(e) => {
            diag::warn!(Ui, "bo2mp hud: {game:?} IWDs not read: {e}");
            Vec::new()
        }
    }
}

fn find_catalog() -> Catalog {
    let mut cat = Catalog::default();
    let iw4 = image_names(asset_core::AssetNamespace::Iw4);
    let pick = |names: &[String], prefix: &str| -> Vec<String> {
        let mut v: Vec<String> = names.iter().filter(|n| n.starts_with(prefix)).cloned().collect();
        v.sort();
        v.dedup();
        v
    };
    cat.cards = pick(&iw4, "cardtitle_");
    // (Emblems: MW2's `cardicon_*`, less its prestige and weapon-perk icons.)
    cat.emblems = pick(&iw4, "cardicon_").into_iter().filter(|n| !n.contains("prestige")).collect();
    diag::info!(
        Ui,
        "bo2mp hud: MW2 IWDs: {} pictures, {} calling cards (cardtitle_*), {} emblems (cardicon_*); first: {:?} / {:?}",
        iw4.len(),
        cat.cards.len(),
        cat.emblems.len(),
        cat.cards.iter().take(6).collect::<Vec<_>>(),
        cat.emblems.iter().take(6).collect::<Vec<_>>()
    );
    cat.titles = mw2_titles();
    // MW3's rank icons (else MW2's).
    for (prefix, ns) in [("iw5", asset_core::AssetNamespace::Iw5), ("iw4", asset_core::AssetNamespace::Iw4)] {
        let names = if prefix == "iw4" { iw4.clone() } else { image_names(ns) };
        let all: Vec<&String> = names.iter().filter(|n| n.starts_with("rank_")).collect();
        let mut ladder: Vec<(usize, u32, String)> =
            all.iter().filter_map(|n| rank_place(n).map(|(i, k)| (i, k, (*n).clone()))).collect();
        ladder.sort();
        ladder.dedup_by(|a, b| a.2 == b.2);
        let other: Vec<&&String> = all.iter().filter(|n| rank_place(n).is_none() && !n.contains("prestige")).collect();
        diag::info!(
            Ui,
            "bo2mp hud: {prefix} rank icons: {} in the ladder {:?}; not placed: {:?}",
            ladder.len(),
            ladder.iter().map(|l| l.2.as_str()).collect::<Vec<_>>(),
            other
        );
        if !ladder.is_empty() {
            cat.ranks = Some((if prefix == "iw5" { "iw5" } else { "iw4" }, ladder.into_iter().map(|l| l.2).collect()));
            break;
        }
    }
    cat
}

/// MW2's own title text (its localized strings), else `MW2_TITLES`.
fn mw2_titles() -> Vec<String> {
    let fallback = || MW2_TITLES.iter().map(|t| (*t).to_owned()).collect::<Vec<_>>();
    let Ok(root) = asset_transport::games_root_from_env() else { return fallback() };
    let mut found: Vec<(String, String)> = Vec::new();
    let mut keys_seen = 0usize;
    let mut sample: Vec<String> = Vec::new();
    for zone in ["iw4:localized_common_mp", "iw4:localized_ui_mp", "iw4:localized_code_post_gfx_mp"] {
        let Ok(file) = asset_transport::find_zone_file(&root, zone) else { continue };
        let Ok(cat) = asset_game::load_localize_catalog(&file.path) else { continue };
        keys_seen += cat.len();
        for (k, v) in cat.iter() {
            let title_key = ["PLAYERCARDS_TITLE_", "CARDTITLE_", "MPUI_CARDTITLE_", "PLAYERCARD_TITLE_"].iter().any(|p| k.starts_with(p));
            if title_key && !v.trim().is_empty() && v.len() <= 28 {
                found.push((k.to_owned(), v.trim().to_owned()));
            } else if k.contains("TITLE") && sample.len() < 24 {
                sample.push(k.to_owned());
            }
        }
    }
    found.sort();
    found.dedup_by(|a, b| a.1 == b.1);
    diag::info!(
        Ui,
        "bo2mp hud: MW2 text: {keys_seen} strings, {} titles found{}; other TITLE keys: {:?}",
        found.len(),
        if found.is_empty() { " (using the built-in MW2 list)" } else { "" },
        sample
    );
    if found.is_empty() { fallback() } else { found.into_iter().map(|(_, v)| v).collect() }
}

/// A player's level (1 first) and prestige: his own from his stats, a
/// player the server gave a rank his, else (bots: no rank of their own) a
/// steady one by name.
fn rank_of(name: &str, meta: Option<&sim::ClientSnapshotMeta>, mine: bool, slot: Option<&crate::lui_hud::LuiHudSlot>) -> (i32, i32) {
    if mine && let Some(r) = slot.and_then(crate::lui_hud::LuiHudSlot::his_rank) {
        return r;
    }
    if let Some(m) = meta.filter(|m| m.rank > 0 || m.prestige > 0) {
        return (m.rank + 1, m.prestige);
    }
    (4 + (name_hash(name, 7) % 52) as i32, 0)
}

/// A rank's icon: MW3's level icon (no prestige), else the combined
/// progression's (BO2's level icons, the other games' prestiges).
fn rank_picture(
    level: i32,
    prestige: i32,
    cat: Option<&Catalog>,
    cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>,
    pics: Option<&mut crate::shell::pics::ShellPics>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    if prestige <= 0
        && let Some((game, ladder)) = cat.and_then(|c| c.ranks.as_ref())
        && !ladder.is_empty()
    {
        let i = (((level.clamp(1, MW3_LEVELS) - 1) as usize) * ladder.len() / MW3_LEVELS as usize).min(ladder.len() - 1);
        let game: &'static str = if *game == "iw5" { "iw5" } else { "iw4" };
        if let Some((h, _)) = game_picture(cache, images, game, &ladder[i]) {
            return Some(h);
        }
    }
    let name = bo2_profile::progression::rank_picture(level, prestige);
    pics?.get(&name, icons, images)
}

// --- Kill feed ----------------------------------------------------------------

/// One row as drawn.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FeedRow {
    pub attacker: String,
    pub attacker_side: Side,
    /// The kill icon and its width per height.
    pub icon: Option<(Handle<Image>, f32)>,
    pub headshot: Option<Handle<Image>>,
    pub victim: String,
    pub victim_side: Side,
    /// When it came (real seconds).
    pub born: f64,
}

#[derive(Component)]
struct FeedRoot;

/// The feed's rows, newest at the bottom, above his name (bottom left).
pub(crate) fn spawn_feed(commands: &mut Commands, faces: &Faces, rows: &[FeedRow], height: f32) -> Entity {
    let k = (height / 1080.0).max(0.4);
    let px = 22.0 * k;
    commands
        .spawn((
            FeedRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(1.2),
                // (Above `hud_modern`'s name and bar.)
                bottom: Val::Px(height * 0.026 + 44.0 * k),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(3.0 * k),
                ..default()
            },
        ))
        .with_children(|col| {
            for r in rows {
                let fade = Fade::new(r.born, FEED_IN, FEED_LIFE, FEED_OUT);
                col.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(7.0 * k),
                        padding: UiRect::new(Val::Px(8.0 * k), Val::Px(12.0 * k), Val::Px(2.0 * k), Val::Px(2.0 * k)),
                        ..default()
                    },
                    BackgroundColor(PANEL.with_alpha(0.42)),
                    fade.with(0.42),
                    FadeBack,
                ))
                .with_children(|row| {
                    let name = |row: &mut ChildSpawnerCommands, s: &str, side: Side| {
                        row.spawn((
                            Text::new(s),
                            font(&faces.bold, px),
                            TextColor(side.name_color()),
                            shadow(px),
                            fade,
                        ));
                    };
                    if !r.attacker.is_empty() {
                        name(row, &r.attacker, r.attacker_side);
                    }
                    let ih = px * 1.05;
                    match &r.icon {
                        Some((image, wide)) => {
                            // (A 4:1 picture is drawn 2:1 tall as BO2 does.)
                            let (w, h) = if *wide >= 4.0 { (ih * 3.2, ih * 0.8) } else { (ih * wide * 1.1, ih) };
                            row.spawn((ImageNode::new(image.clone()), Node { width: Val::Px(w), height: Val::Px(h), ..default() }, fade));
                        }
                        None => {
                            row.spawn((Text::new("—"), font(&faces.regular, px), TextColor(DIM), fade));
                        }
                    }
                    if let Some(hs) = &r.headshot {
                        row.spawn((ImageNode::new(hs.clone()), Node { width: Val::Px(ih), height: Val::Px(ih), ..default() }, fade));
                    }
                    name(row, &r.victim, r.victim_side);
                });
            }
        })
        .id()
}

#[derive(Default)]
struct FeedState {
    last: Option<u32>,
    rows: VecDeque<(u32, FeedRow)>,
    shown: Option<(Vec<u32>, u32)>,
}

#[allow(clippy::too_many_arguments)]
fn kill_feed(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (shell_fonts, game_font): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>),
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<FeedRoot>>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<(Handle<Image>, Vec2)>>>,
    mut st: Local<FeedState>,
) {
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let seen = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) if crate::hud_modern::modern_hud_enabled() => read_seen(p, l, menus.as_deref()),
        _ => None,
    };
    let Some(seen) = seen else {
        clear(&mut commands);
        *st = FeedState::default();
        return;
    };
    let now = now_secs(&time);
    // (Kills from before the first look are old: the last match's may
    // still be in the server's list.)
    let fresh: Vec<Kill> = new_kills(&seen.kills, &mut st.last).into_iter().cloned().collect();
    for k in fresh {
        let icon = icons.as_deref().and_then(|i| kill_icon_name(i, &k.weapon, &k.means));
        let icon = icon.and_then(|(name, wide)| hud_picture(&mut cache, icons.as_deref(), &mut images, &name).map(|(h, _)| (h, wide)));
        let headshot = (k.means == "MOD_HEAD_SHOT" && !k.weapon.is_empty() && k.weapon != "none")
            .then(|| hud_picture(&mut cache, icons.as_deref(), &mut images, "killicon:killiconheadshot:0").map(|(h, _)| h))
            .flatten();
        let row = FeedRow {
            attacker: if k.suicide { String::new() } else { k.attacker.clone() },
            attacker_side: seen.side_of(&k.attacker, &k.attacker_team),
            icon,
            headshot,
            victim: k.victim.clone(),
            victim_side: seen.side_of(&k.victim, &k.victim_team),
            born: now,
        };
        st.rows.push_back((k.n, row));
    }
    st.rows.retain(|(_, r)| ((now - r.born) as f32) < FEED_LIFE);
    while st.rows.len() > FEED_ROWS {
        st.rows.pop_front();
    }
    if !seen.hud || st.rows.is_empty() {
        clear(&mut commands);
        st.shown = None;
        return;
    }
    let height = window.single().map_or(1080.0, Window::height);
    let key = (st.rows.iter().map(|(n, _)| *n).collect::<Vec<_>>(), height as u32);
    if st.shown.as_ref() == Some(&key) && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    clear(&mut commands);
    let rows: Vec<FeedRow> = st.rows.iter().map(|(_, r)| r.clone()).collect();
    spawn_feed(&mut commands, &faces, &rows, height);
    st.shown = Some(key);
}

// --- Hit marker -------------------------------------------------------------

#[derive(Component)]
struct HitRoot;

/// The X at the screen's middle: four short bars around a gap, `age`
/// seconds into its life, red on a kill.
pub(crate) fn spawn_hitmarker(commands: &mut Commands, size: Vec2, age: f32, kill: bool) -> Option<Entity> {
    let life = if kill { KILL_LIFE } else { HIT_LIFE };
    if age >= life {
        return None;
    }
    let k = (size.y / 1080.0).max(0.4);
    // Holds a moment, then fades out; pops in a little bigger.
    let hold = 0.05;
    let alpha = if age < hold { 1.0 } else { (1.0 - (age - hold) / (life - hold)).clamp(0.0, 1.0) };
    let pop = 1.0 + POP * (1.0 - age / POP_TIME).clamp(0.0, 1.0);
    let scale = pop * if kill { 1.12 } else { 1.0 };
    let color = if kill { HIT_RED } else { WHITE };
    let (len, thick, gap) = (12.0 * k * scale, 2.6 * k * scale, 7.5 * k * scale);
    let (cx, cy) = (size.x * 0.5, size.y * 0.5);
    let e = commands
        .spawn((
            HitRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(120),
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ))
        .with_children(|c| {
            for (dx, dy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let r = gap + len * 0.5;
                let (x, y) = (cx + dx * r * std::f32::consts::FRAC_1_SQRT_2, cy + dy * r * std::f32::consts::FRAC_1_SQRT_2);
                let turn = if dx * dy > 0.0 { std::f32::consts::FRAC_PI_4 } else { -std::f32::consts::FRAC_PI_4 };
                // (A soft dark edge under each bar, for bright walls.)
                for (w, h, col) in [(len + 2.0 * k, thick + 2.0 * k, Color::srgba(0.0, 0.0, 0.0, 0.35 * alpha)), (len, thick, color.with_alpha(alpha))] {
                    c.spawn((
                        BackgroundColor(col),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(x - w * 0.5),
                            top: Val::Px(y - h * 0.5),
                            width: Val::Px(w),
                            height: Val::Px(h),
                            ..default()
                        },
                        UiTransform { rotation: Rot2::radians(turn), ..UiTransform::IDENTITY },
                    ));
                }
            }
        })
        .id();
    Some(e)
}

#[derive(Default)]
struct HitState {
    last_kill: Option<u32>,
    alpha: f32,
    last_hit: Option<f64>,
    /// The marker up: when it started, whether it is a kill's.
    marker: Option<(f64, bool)>,
}

#[allow(clippy::too_many_arguments)]
fn hit_marker(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<HitRoot>>,
    time: Res<Time<Real>>,
    mut st: Local<HitState>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    let seen = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) if crate::hud_modern::modern_hud_enabled() => read_seen(p, l, menus.as_deref()),
        _ => None,
    };
    let Some(seen) = seen else {
        *st = HitState::default();
        return;
    };
    let now = now_secs(&time);
    // A hit: BO2's marker element came back up (each hit sets it to 1 and
    // fades it over a second).
    let a = seen.marker_alpha;
    if a > 0.5 && a > st.alpha + 0.02 {
        st.last_hit = Some(now);
        if !st.marker.is_some_and(|(at, kill)| kill && now - at < 0.12) {
            st.marker = Some((now, false));
        }
    }
    st.alpha = a;
    // His kill: the marker turns red (the killing shot's, or a kill with no
    // hit marker of BO2's: a grenade's).
    let kills = new_kills(&seen.kills, &mut st.last_kill);
    if kills.iter().any(|k| seen.is_my_kill(k)) {
        let start = match st.last_hit {
            Some(h) if now - h < KILL_AFTER_HIT => now.min(h + 0.02),
            _ => now,
        };
        st.marker = Some((start.max(now - 0.03), true));
    }
    let Some((at, kill)) = st.marker else { return };
    let age = (now - at) as f32;
    if age >= KILL_LIFE {
        st.marker = None;
        return;
    }
    if !seen.hud {
        return;
    }
    let size = window.single().map_or(Vec2::new(1920.0, 1080.0), |w| Vec2::new(w.width(), w.height()));
    spawn_hitmarker(&mut commands, size, age, kill);
}

// --- "YOU KILLED" banner ------------------------------------------------------

/// The victim's card as drawn.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BannerModel {
    pub name: String,
    pub title: String,
    pub level: i32,
    /// The calling card banner and its picture size.
    pub card: Option<(Handle<Image>, Vec2)>,
    pub emblem: Option<Handle<Image>>,
    pub rank: Option<Handle<Image>>,
    pub start: f64,
}

#[derive(Component)]
struct BannerRoot {
    start: f64,
}

/// The banner under the crosshair: "YOU KILLED" over the victim's card
/// (his emblem, name and title on his calling card, his rank on the right).
pub(crate) fn spawn_banner(commands: &mut Commands, faces: &Faces, m: &BannerModel, height: f32) -> Entity {
    let k = (height / 1080.0).max(0.4);
    let (w, h) = (520.0 * k, 92.0 * k);
    let life = BANNER_IN + BANNER_HOLD + BANNER_OUT;
    let fade = Fade::new(m.start, BANNER_IN, life, BANNER_OUT);
    commands
        .spawn((
            BannerRoot { start: m.start },
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(110),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(64.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0 * k),
                ..default()
            },
            UiTransform::IDENTITY,
        ))
        .with_children(|col| {
            col.spawn((Text::new("YOU KILLED"), font(&faces.bold, 21.0 * k), TextColor(TEXT), shadow(21.0 * k), fade));
            col.spawn((
                Node {
                    width: Val::Px(w),
                    height: Val::Px(h),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(LINE),
                fade.with(0.82),
                FadeBack,
            ))
            .with_children(|card| {
                // The calling card, covering the banner (cut at its edges).
                if let Some((image, size)) = &m.card {
                    let s = crate::shell::pics::cover(*size, Vec2::new(w, h));
                    card.spawn((
                        ImageNode::new(image.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px((w - s.x) * 0.5),
                            top: Val::Px((h - s.y) * 0.5),
                            width: Val::Px(s.x),
                            height: Val::Px(s.y),
                            ..default()
                        },
                        fade,
                    ));
                    // (A navy wash from the left keeps the name readable.)
                    card.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Percent(62.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.025, 0.04, 0.07, 0.55)),
                        fade.with(0.55),
                        FadeBack,
                    ));
                } else {
                    card.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(ACCENT.with_alpha(0.16)),
                        fade.with(0.16),
                        FadeBack,
                    ));
                }
                // The accent along the top.
                card.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Px(2.0 * k),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                    fade,
                    FadeBack,
                ));
                // His emblem.
                let es = h - 14.0 * k;
                card.spawn((
                    Node {
                        width: Val::Px(es),
                        height: Val::Px(es),
                        margin: UiRect::left(Val::Px(7.0 * k)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                    BorderColor::all(LINE),
                    fade.with(0.45),
                    FadeBack,
                ))
                .with_children(|e| match &m.emblem {
                    Some(image) => {
                        e.spawn((ImageNode::new(image.clone()), Node { width: Val::Px(es - 6.0 * k), height: Val::Px(es - 6.0 * k), ..default() }, fade));
                    }
                    None => {
                        let initial: String = m.name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
                        e.spawn((Text::new(initial), font(&faces.bold, 44.0 * k), TextColor(ACCENT), fade));
                    }
                });
                // His name and title.
                card.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::left(Val::Px(14.0 * k)),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|t| {
                    t.spawn((Text::new(m.name.clone()), font(&faces.bold, 34.0 * k), TextColor(WHITE), shadow(34.0 * k), fade));
                    t.spawn((Text::new(m.title.clone()), font(&faces.regular, 20.0 * k), TextColor(TEXT), shadow(20.0 * k), fade));
                });
                // His rank.
                card.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0 * k),
                        padding: UiRect::axes(Val::Px(10.0 * k), Val::Px(6.0 * k)),
                        margin: UiRect::right(Val::Px(10.0 * k)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                    fade.with(0.5),
                    FadeBack,
                ))
                .with_children(|r| {
                    if let Some(icon) = &m.rank {
                        r.spawn((ImageNode::new(icon.clone()), Node { width: Val::Px(38.0 * k), height: Val::Px(38.0 * k), ..default() }, fade));
                    }
                    r.spawn((Text::new(m.level.to_string()), font(&faces.bold, 26.0 * k), TextColor(WHITE), shadow(26.0 * k), fade));
                });
            });
        })
        .id()
}

#[derive(Default)]
struct BannerState {
    last_kill: Option<u32>,
    /// A kill waiting for its banner: the victim and when.
    pending: Option<(String, f64)>,
    shown: Option<(BannerModel, u32)>,
}

/// The banner's pop: a little bigger as it comes in.
fn banner_pop(time: Res<Time<Real>>, mut q: Query<(&BannerRoot, &mut UiTransform)>) {
    let now = now_secs(&time);
    for (b, mut t) in &mut q {
        let age = (now - b.start) as f32;
        let s = 1.0 + 0.06 * (1.0 - age / 0.16).clamp(0.0, 1.0);
        let want = Vec2::splat(s);
        if t.scale != want {
            t.scale = want;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn kill_banner(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (shell_fonts, game_font, mut pics): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>, Option<ResMut<crate::shell::pics::ShellPics>>),
    slot: NonSend<crate::lui_hud::LuiHudSlot>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<BannerRoot>>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<(Handle<Image>, Vec2)>>>,
    mut st: Local<BannerState>,
) {
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let (p, l) = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) if crate::hud_modern::modern_hud_enabled() => (p, l),
        _ => {
            clear(&mut commands);
            *st = BannerState::default();
            return;
        }
    };
    let Some(seen) = read_seen(p, l, menus.as_deref()) else {
        clear(&mut commands);
        *st = BannerState::default();
        return;
    };
    let now = now_secs(&time);
    let cat = catalog();
    for k in new_kills(&seen.kills, &mut st.last_kill) {
        if seen.is_my_kill(k) {
            st.pending = Some((k.victim.clone(), now));
        }
    }
    let snap = p.snapshot();
    let my_score = snap.and_then(|s| s.meta.for_client(l.0)).map_or(0, |m| m.score);
    if let Some((victim, at)) = st.pending.clone()
        && now - at >= BANNER_WAIT
    {
        st.pending = None;
        // (Not the match's last kill: the final killcam and the end screen
        // show that one.)
        if seen.ended || seen.at_limit(my_score) {
            diag::info!(Ui, "bo2mp hud: no kill banner for {victim}: the match ended on it");
        } else {
            let meta = snap.and_then(|s| {
                s.meta
                    .clients
                    .iter()
                    .find(|(_, m)| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)) == victim)
                    .map(|(_, m)| m.clone())
            });
            let (level, prestige) = rank_of(&victim, meta.as_ref(), false, Some(&*slot));
            let title = cat
                .as_ref()
                .map(|c| c.titles.clone())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| MW2_TITLES.iter().map(|t| (*t).to_owned()).collect());
            let title = title[name_hash(&victim, 1) as usize % title.len()].clone();
            let model = BannerModel {
                name: victim.clone(),
                title,
                level,
                card: None,
                emblem: None,
                rank: None,
                start: now,
            };
            diag::info!(Ui, "bo2mp hud: kill banner: {victim} \"{}\" level {level} prestige {prestige}", model.title);
            st.shown = Some((model, 0));
        }
    }
    let life = (BANNER_IN + BANNER_HOLD + BANNER_OUT) as f64;
    if st.shown.as_ref().is_some_and(|(m, _)| now - m.start >= life) {
        st.shown = None;
    }
    let Some((mut model, built)) = st.shown.clone() else {
        clear(&mut commands);
        return;
    };
    if !seen.hud {
        clear(&mut commands);
        st.shown = Some((model, 0));
        return;
    }
    // Its pictures (read from MW2's files; in as they come).
    if let Some(c) = cat.as_ref() {
        if !c.cards.is_empty() {
            let name = &c.cards[name_hash(&model.name, 2) as usize % c.cards.len()];
            model.card = game_picture(&mut cache, &mut images, "iw4", name);
        }
        if !c.emblems.is_empty() {
            let name = &c.emblems[name_hash(&model.name, 3) as usize % c.emblems.len()];
            model.emblem = game_picture(&mut cache, &mut images, "iw4", name).map(|(h, _)| h);
        }
    }
    let meta = snap.and_then(|s| {
        s.meta
            .clients
            .iter()
            .find(|(_, m)| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)) == model.name)
            .map(|(_, m)| m.clone())
    });
    let (level, prestige) = rank_of(&model.name, meta.as_ref(), false, Some(&*slot));
    model.rank = rank_picture(level, prestige, cat.as_ref(), &mut cache, pics.as_deref_mut(), icons.as_deref(), &mut images);
    let height = window.single().map_or(1080.0, Window::height);
    // Built again when a picture comes in or the window changes.
    let key = u32::from(model.card.is_some()) | u32::from(model.emblem.is_some()) << 1 | u32::from(model.rank.is_some()) << 2 | (height as u32) << 3;
    if built == key && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    clear(&mut commands);
    spawn_banner(&mut commands, &faces, &model, height);
    st.shown = Some((model, key));
}

// --- Scoreboard ---------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoardRow {
    pub name: String,
    pub level: i32,
    pub rank: Option<Handle<Image>>,
    pub score: i32,
    pub kills: i32,
    pub assists: i32,
    pub deaths: i32,
    pub ping: i32,
    pub me: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoardGroup {
    /// The side's name and score (none: a free-for-all's one list).
    pub title: Option<(String, i32)>,
    pub side: Side,
    pub emblem: Option<Handle<Image>>,
    pub rows: Vec<BoardRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BoardModel {
    pub mode: String,
    pub clock: Option<String>,
    pub groups: Vec<BoardGroup>,
}

#[derive(Component)]
struct BoardRoot;

const COLUMNS: [(&str, f32); 6] = [("SCORE", 96.0), ("KILLS", 84.0), ("ASSISTS", 90.0), ("DEATHS", 84.0), ("K/D RATIO", 96.0), ("PING", 78.0)];

/// The scoreboard in the screen's middle.
pub(crate) fn spawn_board(commands: &mut Commands, faces: &Faces, m: &BoardModel, height: f32) -> Entity {
    let k = (height / 1080.0).max(0.4);
    let width = 1000.0 * k;
    let row_h = 34.0 * k;
    let cells = |r: &mut ChildSpawnerCommands, values: [String; 6], color: Color, f: &Handle<Font>, px: f32| {
        for ((_, w), v) in COLUMNS.iter().zip(values) {
            r.spawn(Node { width: Val::Px(w * k), justify_content: JustifyContent::Center, ..default() })
                .with_children(|c| {
                    c.spawn((Text::new(v), font(f, px), TextColor(color)));
                });
        }
    };
    commands
        .spawn((
            BoardRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(14_000),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Percent(14.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|screen| {
            screen
                .spawn((
                    Node {
                        width: Val::Px(width),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(LINE),
                ))
                .with_children(|panel| {
                    // The accent along the top.
                    panel.spawn((BackgroundColor(ACCENT), Node { width: Val::Percent(100.0), height: Val::Px(3.0 * k), ..default() }));
                    // Header: the game type, the clock.
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::FlexEnd,
                            padding: UiRect::new(Val::Px(18.0 * k), Val::Px(18.0 * k), Val::Px(10.0 * k), Val::Px(10.0 * k)),
                            ..default()
                        })
                        .with_children(|h| {
                            h.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|c| {
                                c.spawn((Text::new("GAMETYPE"), font(&faces.regular, 16.0 * k), TextColor(FAINT)));
                                c.spawn((Text::new(m.mode.clone()), font(&faces.bold, 40.0 * k), TextColor(WHITE)));
                            });
                            h.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexEnd, ..default() })
                                .with_children(|c| {
                                    c.spawn((Text::new("TIME"), font(&faces.regular, 16.0 * k), TextColor(FAINT)));
                                    c.spawn((Text::new(m.clock.clone().unwrap_or_else(|| "--:--".to_owned())), font(&faces.bold, 40.0 * k), TextColor(WHITE)));
                                });
                        });
                    // Column headers.
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                height: Val::Px(26.0 * k),
                                padding: UiRect::horizontal(Val::Px(14.0 * k)),
                                border: UiRect::vertical(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                            BorderColor::all(LINE),
                        ))
                        .with_children(|r| {
                            r.spawn(Node { flex_grow: 1.0, ..default() }).with_children(|c| {
                                c.spawn((Text::new("PLAYER"), font(&faces.bold, 15.0 * k), TextColor(DIM)));
                            });
                            cells(r, COLUMNS.map(|(n, _)| n.to_owned()), DIM, &faces.bold, 15.0 * k);
                        });
                    for g in &m.groups {
                        if let Some((title, score)) = &g.title {
                            let tint = match g.side {
                                Side::Mine => ACCENT,
                                Side::Enemy => ENEMY,
                                _ => DIM,
                            };
                            panel
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(10.0 * k),
                                        height: Val::Px(32.0 * k),
                                        padding: UiRect::horizontal(Val::Px(14.0 * k)),
                                        margin: UiRect::top(Val::Px(6.0 * k)),
                                        border: UiRect::left(Val::Px(3.0 * k)),
                                        ..default()
                                    },
                                    crate::shell::theme::fade_right(tint.with_alpha(0.30), tint.with_alpha(0.0)),
                                    BorderColor::all(tint),
                                ))
                                .with_children(|r| {
                                    if let Some(e) = &g.emblem {
                                        r.spawn((ImageNode::new(e.clone()), Node { width: Val::Px(26.0 * k), height: Val::Px(26.0 * k), ..default() }));
                                    }
                                    r.spawn((Text::new(title.clone()), font(&faces.bold, 22.0 * k), TextColor(WHITE)));
                                    r.spawn(Node { flex_grow: 1.0, ..default() });
                                    r.spawn((Text::new(score.to_string()), font(&faces.bold, 24.0 * k), TextColor(WHITE)));
                                });
                        }
                        for (i, row) in g.rows.iter().enumerate() {
                            let back = match g.side {
                                Side::Mine => MINE_ROW,
                                Side::Enemy => ENEMY_ROW,
                                _ if row.me => MINE_ROW,
                                _ if i % 2 == 0 => PLAIN_ROW,
                                _ => Color::NONE,
                            };
                            let name_color = if row.me { GOLD } else { WHITE };
                            let text_color = if row.me { GOLD } else { TEXT };
                            panel
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        height: Val::Px(row_h),
                                        padding: UiRect::horizontal(Val::Px(14.0 * k)),
                                        margin: UiRect::top(Val::Px(2.0 * k)),
                                        ..default()
                                    },
                                    BackgroundColor(back),
                                ))
                                .with_children(|r| {
                                    r.spawn(Node {
                                        flex_grow: 1.0,
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(8.0 * k),
                                        ..default()
                                    })
                                    .with_children(|p| {
                                        match &row.rank {
                                            Some(icon) => {
                                                p.spawn((ImageNode::new(icon.clone()), Node { width: Val::Px(26.0 * k), height: Val::Px(26.0 * k), ..default() }));
                                            }
                                            None => {
                                                p.spawn((BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)), Node { width: Val::Px(22.0 * k), height: Val::Px(22.0 * k), margin: UiRect::horizontal(Val::Px(2.0 * k)), ..default() }));
                                            }
                                        }
                                        p.spawn(Node { width: Val::Px(40.0 * k), ..default() }).with_children(|c| {
                                            c.spawn((Text::new(row.level.to_string()), font(&faces.regular, 19.0 * k), TextColor(DIM)));
                                        });
                                        p.spawn((Text::new(row.name.clone()), font(&faces.bold, 21.0 * k), TextColor(name_color)));
                                    });
                                    let ratio = if row.deaths > 0 { row.kills as f32 / row.deaths as f32 } else { row.kills as f32 };
                                    cells(
                                        r,
                                        [
                                            row.score.to_string(),
                                            row.kills.to_string(),
                                            row.assists.to_string(),
                                            row.deaths.to_string(),
                                            format!("{ratio:.2}"),
                                            row.ping.to_string(),
                                        ],
                                        text_color,
                                        &faces.regular,
                                        20.0 * k,
                                    );
                                });
                        }
                    }
                    panel.spawn(Node { height: Val::Px(10.0 * k), ..default() });
                });
        })
        .id()
}

/// The game type's name when BO2's tables do not give it.
fn mode_name(gametype: &str) -> String {
    match gametype {
        "tdm" => "TEAM DEATHMATCH",
        "dm" => "FREE-FOR-ALL",
        "dom" => "DOMINATION",
        "sd" => "SEARCH & DESTROY",
        "ctf" => "CAPTURE THE FLAG",
        "koth" => "HARDPOINT",
        "hq" => "HEADQUARTERS",
        "dem" => "DEMOLITION",
        "conf" => "KILL CONFIRMED",
        "gun" => "GUN GAME",
        "oic" => "ONE IN THE CHAMBER",
        "sas" => "STICKS AND STONES",
        "shrp" => "SHARPSHOOTER",
        other => return other.to_ascii_uppercase(),
    }
    .to_owned()
}

#[derive(Default)]
struct BoardState {
    shown: Option<(BoardModel, u32)>,
    end_logged: bool,
}

#[allow(clippy::too_many_arguments)]
fn scoreboard(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (shell_fonts, game_font, mut pics): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>, Option<ResMut<crate::shell::pics::ShellPics>>),
    slot: NonSend<crate::lui_hud::LuiHudSlot>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<BoardRoot>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<(Handle<Image>, Vec2)>>>,
    mut st: Local<BoardState>,
) {
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let up = crate::hud_modern::modern_hud_enabled() && slot.board_wanted();
    let (Some(p), Some(l), true) = (presented.as_deref(), local.as_deref(), up) else {
        clear(&mut commands);
        st.shown = None;
        st.end_logged = false;
        return;
    };
    let Some(seen) = read_seen(p, l, None) else {
        clear(&mut commands);
        st.shown = None;
        return;
    };
    let Some(snap) = p.snapshot() else { return };
    let cat = catalog();
    let me = l.0.0;
    let mut groups: Vec<BoardGroup> = Vec::new();
    let row_of = |c: u32, m: &sim::ClientSnapshotMeta, cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>, images: &mut Assets<Image>, pics: Option<&mut crate::shell::pics::ShellPics>| {
        let name = String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned();
        let mine = c == me;
        let (level, prestige) = rank_of(&name, Some(m), mine, Some(&*slot));
        let rank = rank_picture(level, prestige, cat.as_ref(), cache, pics, icons.as_deref(), images);
        // (Ping as BO2's own board: the listen server's client 12-16 ms,
        // bots 0.)
        let ping = if mine { 12 + (name.bytes().map(u32::from).sum::<u32>() % 5) as i32 } else { 0 };
        BoardRow { name, level, rank, score: m.score, kills: m.kills, assists: 0, deaths: m.deaths, ping, me: mine }
    };
    let mut players: Vec<(u32, &sim::ClientSnapshotMeta, String)> = snap
        .meta
        .clients
        .iter()
        .map(|(c, m)| (c.0, m, seen.teams.get(&c.0).cloned().unwrap_or_default()))
        .filter(|(_, _, t)| !matches!(t.as_str(), "spectator" | ""))
        .collect();
    players.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(b.1.kills.cmp(&a.1.kills)).then(a.1.deaths.cmp(&b.1.deaths)));
    let mut emblem = |name: &str| game_picture(&mut cache, &mut images, "t5", name).map(|(h, _)| h);
    let emblems = (emblem("faction_128_specops"), emblem("faction_128_spetsnaz"));
    if seen.ffa {
        let rows = players.iter().map(|(c, m, _)| row_of(*c, m, &mut cache, &mut images, pics.as_deref_mut())).collect();
        groups.push(BoardGroup { title: None, side: Side::Neutral, emblem: None, rows });
    } else {
        let mine = if seen.my_team == "axis" { "axis" } else { "allies" };
        for team in [mine, if mine == "axis" { "allies" } else { "axis" }] {
            let rows: Vec<BoardRow> = players
                .iter()
                .filter(|(_, _, t)| t == team)
                .map(|(c, m, _)| row_of(*c, m, &mut cache, &mut images, pics.as_deref_mut()))
                .collect();
            let (title, score, em) = if team == "axis" {
                ("SPETSNAZ", seen.scores.1, emblems.1.clone())
            } else {
                ("BLACK OPS", seen.scores.0, emblems.0.clone())
            };
            groups.push(BoardGroup {
                title: Some((title.to_owned(), score)),
                side: if team == mine { Side::Mine } else { Side::Enemy },
                emblem: em,
                rows,
            });
        }
    }
    let model = BoardModel {
        mode: slot.gametype_title(&seen.gametype).unwrap_or_else(|| mode_name(&seen.gametype)),
        clock: seen.timeleft.map(|t| format!("{}:{:02}", t / 60, t % 60)),
        groups,
    };
    let height = window.single().map_or(1080.0, Window::height);
    let key = (model.clone(), height as u32);
    if st.shown.as_ref() == Some(&key) && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    clear(&mut commands);
    spawn_board(&mut commands, &faces, &model, height);
    // (The gate's proof the end scoreboard drew its rows, as BO2's board
    // logged it.)
    if seen.ended && !st.end_logged {
        let n: usize = model.groups.iter().map(|g| g.rows.len()).sum();
        if n > 0 {
            diag::info!(Ui, "bo2mp end scoreboard: rows drawn {n} of {n}");
            st.end_logged = true;
        }
    }
    st.shown = Some(key);
}

pub(crate) fn register_match_hud_systems(app: &mut App) {
    app.add_systems(
        Update,
        (kill_feed, hit_marker, kill_banner, scoreboard).in_set(ClientSet::Ui),
    )
    .add_systems(Update, (banner_pop, apply_fades).in_set(ClientSet::Ui).after(kill_banner).after(kill_feed));
}

// --- Preview --------------------------------------------------------------------

pub mod preview {
    //! The match HUD's pieces with stand-in data, saved as pictures
    //! (`cargo run -p ui --example hud_preview -- <out dir>`).

    use std::path::PathBuf;

    use bevy::prelude::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::*;

    #[derive(Resource)]
    struct Plan {
        out: PathBuf,
        step: usize,
        frames: u32,
    }

    #[derive(Component)]
    struct Shot;

    /// A stand-in picture: a soft diagonal gradient (a calling card), or a
    /// chevron (a rank icon), or a gun's outline.
    fn stand_in(kind: u8, w: u32, h: u32) -> Image {
        use bevy::asset::RenderAssetUsages;
        use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
                let o = ((y * w + x) * 4) as usize;
                let c: [f32; 4] = match kind {
                    0 => {
                        let s = (u * 6.0 + v * 2.0).sin() * 0.5 + 0.5;
                        [0.55 + 0.35 * s, 0.10 + 0.1 * v, 0.08, 1.0]
                    }
                    1 => {
                        let d = (u - 0.5).abs();
                        let on = (v - 0.25 - d * 0.8).abs() < 0.09 || (v - 0.55 - d * 0.8).abs() < 0.09;
                        if on { [0.85, 0.82, 0.70, 1.0] } else { [0.0; 4] }
                    }
                    2 => {
                        let body = v > 0.3 && v < 0.55 && u > 0.05 && u < 0.95;
                        let grip = v >= 0.55 && v < 0.95 && u > 0.55 && u < 0.68;
                        let mag = v >= 0.55 && v < 0.85 && u > 0.35 && u < 0.45;
                        if body || grip || mag { [1.0, 1.0, 1.0, 1.0] } else { [0.0; 4] }
                    }
                    _ => {
                        let r = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt();
                        if r < 0.42 { [0.20, 0.30, 0.45, 1.0] } else { [0.0; 4] }
                    }
                };
                for k in 0..4 {
                    px[o + k] = (c[k] * 255.0) as u8;
                }
            }
        }
        Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())
    }

    fn step(
        mut commands: Commands,
        mut plan: ResMut<Plan>,
        fonts: Option<Res<ShellFonts>>,
        shots: Query<Entity, With<Shot>>,
        time: Res<Time<Real>>,
        mut images: ResMut<Assets<Image>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let Some(fonts) = fonts else { return };
        let faces = Faces { regular: fonts.regular.clone(), bold: fonts.bold.clone() };
        plan.frames += 1;
        if plan.frames < 30 {
            return;
        }
        plan.frames = 0;
        let i = plan.step / 2;
        let names = ["01_feed_banner_hit", "02_kill_hitmarker", "03_scoreboard_teams", "04_scoreboard_ffa"];
        if i >= names.len() {
            exit.write(AppExit::Success);
            return;
        }
        if plan.step % 2 == 1 {
            let path = plan.out.join(format!("{}.png", names[i]));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            plan.step += 1;
            return;
        }
        for e in &shots {
            commands.entity(e).despawn();
        }
        // (Pieces are shown in full: their fades are set as if just in.)
        let now = time.elapsed_secs_f64() - 0.5;
        let gun = images.add(stand_in(2, 128, 64));
        let wide_gun = images.add(stand_in(2, 256, 64));
        let rank = images.add(stand_in(1, 64, 64));
        let emblem = images.add(stand_in(3, 64, 64));
        let card = images.add(stand_in(0, 512, 96));
        let size = Vec2::new(1920.0, 1080.0);
        let mark = |commands: &mut Commands, e: Entity| {
            commands.entity(e).insert(Shot);
        };
        match i {
            0 | 1 => {
                let rows = vec![
                    FeedRow { attacker: "Jebus".into(), attacker_side: Side::Mine, icon: Some((gun.clone(), 2.0)), headshot: None, victim: "Jimbo".into(), victim_side: Side::Enemy, born: now },
                    FeedRow { attacker: "RezTech".into(), attacker_side: Side::Enemy, icon: Some((wide_gun.clone(), 4.0)), headshot: Some(emblem.clone()), victim: "eden".into(), victim_side: Side::Mine, born: now },
                    FeedRow { attacker: "simon".into(), attacker_side: Side::You, icon: Some((gun.clone(), 2.0)), headshot: None, victim: "MjrDeathAdder".into(), victim_side: Side::Enemy, born: now },
                    FeedRow { attacker: "Sass".into(), attacker_side: Side::Enemy, icon: Some((wide_gun.clone(), 4.0)), headshot: None, victim: "simon".into(), victim_side: Side::You, born: now },
                    FeedRow { attacker: String::new(), attacker_side: Side::Neutral, icon: Some((emblem.clone(), 1.0)), headshot: None, victim: "JTAG".into(), victim_side: Side::Enemy, born: now },
                ];
                let e = spawn_feed(&mut commands, &faces, &rows, 1080.0);
                mark(&mut commands, e);
                let model = BannerModel {
                    name: "Jimbo".into(),
                    title: "Gun Runner".into(),
                    level: 62,
                    card: Some((card.clone(), Vec2::new(512.0, 96.0))),
                    emblem: Some(emblem.clone()),
                    rank: Some(rank.clone()),
                    start: now,
                };
                let e = spawn_banner(&mut commands, &faces, &model, 1080.0);
                mark(&mut commands, e);
                if let Some(e) = spawn_hitmarker(&mut commands, size, 0.02, i == 1) {
                    mark(&mut commands, e);
                }
                // A dim stand-in scene behind.
                let e = commands.spawn((BackgroundColor(Color::srgb(0.30, 0.31, 0.29)), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(-1))).id();
                mark(&mut commands, e);
            }
            _ => {
                let row = |name: &str, level: i32, score: i32, kills: i32, deaths: i32, me: bool| BoardRow {
                    name: name.into(),
                    level,
                    rank: Some(rank.clone()),
                    score,
                    kills,
                    assists: kills / 4,
                    deaths,
                    ping: if me { 14 } else { 0 },
                    me,
                };
                let groups = if i == 2 {
                    vec![
                        BoardGroup {
                            title: Some(("BLACK OPS".into(), 4200)),
                            side: Side::Mine,
                            emblem: Some(emblem.clone()),
                            rows: vec![row("eden", 31, 1100, 12, 5, false), row("simon", 47, 700, 10, 7, true), row("Jebus", 12, 600, 8, 7, false)],
                        },
                        BoardGroup {
                            title: Some(("SPETSNAZ".into(), 3800)),
                            side: Side::Enemy,
                            emblem: Some(emblem.clone()),
                            rows: vec![row("BlueGhost", 50, 700, 10, 6, false), row("MjrDeathAdder", 9, 300, 4, 7, false), row("JTAG", 55, 100, 5, 7, false)],
                        },
                    ]
                } else {
                    vec![BoardGroup {
                        title: None,
                        side: Side::Neutral,
                        emblem: None,
                        rows: vec![
                            row("eden", 31, 1100, 12, 5, false),
                            row("BlueGhost", 50, 700, 10, 6, false),
                            row("simon", 47, 700, 10, 7, true),
                            row("Jebus", 12, 600, 8, 7, false),
                            row("MjrDeathAdder", 9, 300, 4, 7, false),
                            row("JTAG", 55, 100, 5, 7, false),
                            row("Fish_Sdicks", 22, 0, 3, 7, false),
                            row("Sass", 3, 0, 1, 7, false),
                        ],
                    }]
                };
                let model = BoardModel {
                    mode: if i == 2 { "TEAM DEATHMATCH".into() } else { "GUN GAME".into() },
                    clock: Some("9:15".into()),
                    groups,
                };
                let e = spawn_board(&mut commands, &faces, &model, 1080.0);
                mark(&mut commands, e);
                let e = commands.spawn((BackgroundColor(Color::srgb(0.30, 0.31, 0.29)), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, GlobalZIndex(-1))).id();
                mark(&mut commands, e);
            }
        }
        plan.step += 1;
    }

    /// (The preview's frames are slow: every piece is held half a second
    /// into its life, fully in.)
    fn hold_fades(time: Res<Time<Real>>, mut q: Query<&mut Fade>) {
        let now = time.elapsed_secs_f64();
        for mut f in &mut q {
            f.start = now - 0.5;
        }
    }

    /// The pieces with stand-in data, a picture of each saved in `out`.
    pub fn install(app: &mut App, out: PathBuf) {
        app.add_systems(Startup, crate::shell::theme::load_fonts);
        app.insert_resource(Plan { out, step: 0, frames: 0 });
        app.add_systems(Update, (step, hold_fades, super::apply_fades, super::banner_pop).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_lines_parse() {
        let k = parse_feed("3|Jebus|Jimbo|an94_mp+reflex|MOD_HEAD_SHOT||allies|axis;4|Sass|Sass|none|MOD_SUICIDE|self|axis|axis");
        assert_eq!(k.len(), 2);
        assert_eq!(k[0].attacker, "Jebus");
        assert_eq!(k[0].means, "MOD_HEAD_SHOT");
        assert!(k[1].suicide);
    }

    #[test]
    fn new_kills_once_and_reset() {
        let feed = parse_feed("1|a|b|x|y;2|a|c|x|y");
        let mut last = None;
        assert!(new_kills(&feed, &mut last).is_empty());
        let more = parse_feed("1|a|b|x|y;2|a|c|x|y;3|a|d|x|y");
        assert_eq!(new_kills(&more, &mut last).len(), 1);
        assert!(new_kills(&more, &mut last).is_empty());
        // A new match counts from 1 again.
        let fresh = parse_feed("1|z|y|x|y");
        assert_eq!(new_kills(&fresh, &mut last).len(), 1);
    }

    #[test]
    fn rank_names_place() {
        assert_eq!(rank_place("rank_pvt1"), Some((0, 1)));
        assert_eq!(rank_place("rank_1stsgt2"), Some((8, 2)));
        assert_eq!(rank_place("rank_comm"), Some((21, 0)));
        assert_eq!(rank_place("rank_prestige3"), None);
        assert!(rank_place("rank_sgt1").unwrap().0 < rank_place("rank_ssgt1").unwrap().0);
    }

    #[test]
    fn hits_fade_and_kills_last_longer() {
        let f = Fade::new(0.0, 0.2, 6.0, 0.6);
        assert!(f.at(0.0) < 0.01);
        assert!((f.at(3.0) - 1.0).abs() < 1e-6);
        assert!(f.at(6.1) < 0.01);
        const { assert!(KILL_LIFE > HIT_LIFE) };
    }
}
