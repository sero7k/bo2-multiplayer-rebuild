//! bo2mp: the match's own kill feed, hit marker, kill and Killed By cards and
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
//! - Kill card: the victim's card at the lower middle, under the gun (its
//!   middle 82% down, 310 x 84 at 1080 lines): his Modern Warfare 2 / 3
//!   calling card with his emblem on the left (`hud_cards`; a plain drawn
//!   card until or unless the picture is in, never Black Ops II's), his
//!   name at the top, his rank icon (Black Ops' `rank_*` icons, read from
//!   its IWDs) and level small at the bottom right; it rises and settles
//!   in, holds and fades. No sound, as in Black Ops. Not for the match's
//!   last kill (the final killcam has it).
//! - Killed By: at his death, before the killcam, the killer's card the
//!   same way under "KILLED BY" (BO2's own LUI card is not sent).
//! - Killcam card: through the killcam, the killer's card at the bottom
//!   middle where BO2's Killcam widget had its NamePlate (that one is not
//!   made; the widget's title, bands and prompts stay).
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

/// The kill card's fades and hold (s), and how long a kill waits before
/// its card (the match may have ended on it).
const BANNER_IN: f32 = 0.15;
const BANNER_HOLD: f32 = 2.4;
const BANNER_OUT: f32 = 0.4;
const BANNER_WAIT: f64 = 0.22;
/// The card's slide up (px at 1080) and pop as it comes in, and how long.
const CARD_RISE: f32 = 26.0;
const CARD_POP: f32 = 0.05;
const CARD_SETTLE: f32 = 0.22;
/// Black Ops played no sound with its kill card (its `playerCardPopUp`
/// menu plays none, no script plays one with it, and its sound zones have
/// no card alias): the kill is heard in the hit marker.
const KILL_CARD_SOUND: Option<&str> = None;

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
    /// A panel's edge's alpha (`BorderColor`, with `FadeEdge`).
    edge: f32,
}

impl Fade {
    fn new(start: f64, fade_in: f32, life: f32, out: f32) -> Self {
        Fade { start, fade_in, life, out, alpha: 1.0, shadow: 0.75, edge: 0.85 }
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

/// A `Fade` that fades the node's edge (`BorderColor`) too.
#[derive(Component, Clone, Copy, Debug)]
struct FadeEdge;

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
        Option<&mut BorderColor>,
        Has<FadeBack>,
        Has<FadeEdge>,
    )>,
) {
    let now = now_secs(&time);
    for (f, text, image, back, shade, edge, fades_back, fades_edge) in &mut q {
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
        // (A panel's edge fades with it, from its own alpha.)
        if let Some(mut e) = edge.filter(|_| fades_edge) {
            let want = BorderColor::all(e.top.with_alpha(f.edge * a));
            if *e != want {
                *e = want;
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

// --- Black Ops' card pictures and Modern Warfare 3's rank icons -------------

/// What another game's files give the kill card and the rank icons, found
/// once off the main thread (each empty when its game is not there).
#[derive(Clone, Debug, Default)]
struct Catalog {
    /// Black Ops' rank icons in its IWDs, one per `BO1_RANKS` row (`None`:
    /// none of that row's names is there).
    bo1_ranks: Vec<Option<String>>,
    /// MW3's level icons in rank order (`rank_*` images): the game and its
    /// pictures (the scoreboard's).
    ranks: Option<(&'static str, Vec<String>)>,
}

/// Black Ops' ranks: the level each starts at and its icon's image names
/// (its `rank_<x>_128` materials' images, as its `ui_mp` / `common_mp`
/// zones name them: `rank_prvt_128` draws `rank_pvt1_128`, ...), first the
/// image, then the material names. 50 levels; its General and Commander
/// have one level each (no `RANK_GEN_FULL_N` / `RANK_COM_FULL_N`).
const BO1_RANKS: &[(i32, &[&str])] = &[
    (1, &["rank_pvt1_128", "rank_prvt_128", "rank_prvt"]),
    (4, &["rank_pfc1_128", "rank_pfc_128", "rank_pfc"]),
    (7, &["rank_spc1_128", "rank_spc_128", "rank_spc"]),
    (10, &["rank_cpl1_128", "rank_cpl_128", "rank_cpl"]),
    (13, &["rank_sgt1_128", "rank_sgt_128", "rank_sgt"]),
    (16, &["rank_ssgt1_128", "rank_ssgt_128", "rank_ssgt"]),
    (19, &["rank_sgtfc1_128", "rank_sfc_128", "rank_sfc"]),
    (22, &["rank_sgtm1_128", "rank_sgtm_128", "rank_sgtm"]),
    (25, &["rank_2ndlt1_128", "rank_lt_128", "rank_lt"]),
    (28, &["rank_capt1_128", "rank_cpt_128", "rank_cpt"]),
    (31, &["rank_maj1_128", "rank_maj_128", "rank_maj"]),
    (34, &["rank_ltcol1_128", "rank_ltcol_128", "rank_ltcol"]),
    (37, &["rank_col1_128", "rank_col_128", "rank_col"]),
    (40, &["rank_bgen1_128", "rank_briggen_128", "rank_briggen"]),
    (43, &["rank_majgen1_128", "rank_majgen_128", "rank_majgen"]),
    (46, &["rank_ltgen1_128", "rank_ltgen_128", "rank_ltgen"]),
    (49, &["rank_gen1_128", "rank_gen_128", "rank_gen"]),
    (50, &["rank_comm_128", "rank_com_128", "rank_com"]),
];

/// Black Ops' row in `BO1_RANKS` for a level (its last for any higher).
fn bo1_rank_row(level: i32) -> usize {
    BO1_RANKS.iter().rposition(|(first, _)| level.max(1) >= *first).unwrap_or(0)
}

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
    // Black Ops' pictures (its IWDs): its rank icons.
    let t5 = image_names(asset_core::AssetNamespace::T5);
    let has = |n: &str| t5.iter().any(|x| x.eq_ignore_ascii_case(n));
    cat.bo1_ranks = BO1_RANKS.iter().map(|(_, names)| names.iter().find(|n| has(n)).map(|n| (*n).to_owned())).collect();
    let like = |words: &[&str]| -> Vec<&String> {
        let mut v: Vec<&String> = t5.iter().filter(|n| words.iter().any(|w| n.to_ascii_lowercase().contains(w))).collect();
        v.sort();
        v.dedup();
        v
    };
    diag::info!(
        Ui,
        "bo2mp hud: BO1 IWDs: {} pictures; rank icons {:?}; BO1 rank_* {:?}",
        t5.len(),
        cat.bo1_ranks,
        like(&["rank_"]).iter().take(80).collect::<Vec<_>>(),
    );
    // MW3's rank icons (else MW2's), for the scoreboard.
    for (prefix, ns) in [("iw5", asset_core::AssetNamespace::Iw5), ("iw4", asset_core::AssetNamespace::Iw4)] {
        let names = image_names(ns);
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

// --- The kill card and the Killed By card -----------------------------------

/// A player's card as drawn: his Modern Warfare 2 / 3 calling card with his
/// emblem on it, his name, his rank icon and level.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BannerModel {
    pub name: String,
    pub level: i32,
    /// His rank icon (Black Ops', else the combined progression's).
    pub rank: Option<Handle<Image>>,
    /// His calling card (`cardtitle_*`) and its picture size; none: the
    /// plain drawn card.
    pub card: Option<(Handle<Image>, Vec2)>,
    /// His emblem (`cardicon_*`).
    pub emblem: Option<Handle<Image>>,
    /// A line over the card ("KILLED BY").
    pub title: Option<String>,
    /// His kill streak this life, in a red box over the card ("2ND KILL",
    /// as Modern Warfare's kill card).
    pub streak: Option<String>,
    pub start: f64,
    /// How long it holds before it fades (s).
    pub hold: f32,
}

impl BannerModel {
    pub(crate) fn new(name: String, level: i32, start: f64) -> Self {
        BannerModel { name, level, rank: None, card: None, emblem: None, title: None, streak: None, start, hold: BANNER_HOLD }
    }
}

#[derive(Component)]
struct BannerRoot {
    start: f64,
}

/// The Killed By card (a `BannerRoot` too, for its slide in).
#[derive(Component)]
struct KilledByRoot;

/// The card's edge, the plain drawn card's two ends, the shade over a
/// calling card that keeps the name readable.
const CARD_EDGE: Color = Color::srgba(0.62, 0.66, 0.72, 0.85);
const CARD_TOP: Color = Color::srgba(0.16, 0.20, 0.27, 0.35);
const CARD_FOOT: Color = Color::srgba(0.03, 0.04, 0.06, 0.94);
const CARD_SHADE: Color = Color::srgba(0.0, 0.0, 0.0, 0.30);
const EMBLEM_BACK: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);

/// The card on a 1080-line screen (about 72% of the old Black Ops card,
/// 430 x 116) and where its middle sits: 82% of the way down, under the
/// gun, clear of the kill feed (bottom left) and the ammo corner (bottom
/// right).
const CARD_W: f32 = 310.0;
const CARD_H: f32 = 84.0;
const CARD_CENTRE: f32 = 0.82;

/// The card's scale on a `size` screen: by its height, as the rest of the
/// HUD, but no wider than a share of a narrow screen.
fn card_scale(size: Vec2) -> f32 {
    (size.y / 1080.0).min(size.x / 1400.0).max(0.4)
}

/// The card's place: its top edge in pixels.
fn card_top(size: Vec2) -> f32 {
    (size.y * CARD_CENTRE - CARD_H * card_scale(size) * 0.5).round()
}

/// The card at the lower middle of the screen: his calling card covering
/// it (shaded a little), his emblem on the left, his name at the top and
/// his rank icon and level small at the bottom right; a title over it.
pub(crate) fn spawn_banner(commands: &mut Commands, faces: &Faces, m: &BannerModel, size: Vec2) -> Entity {
    let k = card_scale(size);
    let (w, h) = ((CARD_W * k).round(), (CARD_H * k).round());
    let life = BANNER_IN + m.hold + BANNER_OUT;
    let fade = Fade::new(m.start, BANNER_IN, life, BANNER_OUT);
    commands
        .spawn((
            BannerRoot { start: m.start },
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(110),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(card_top(size)),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTransform::IDENTITY,
        ))
        .with_children(|col| {
            col.spawn((
                Node {
                    width: Val::Px(w),
                    height: Val::Px(h),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px((1.0 * k).max(1.0))),
                    ..default()
                },
                BackgroundColor(CARD_FOOT),
                BorderColor::all(CARD_EDGE),
                fade.with(CARD_FOOT.alpha()),
                FadeBack,
                FadeEdge,
            ))
            .with_children(|card| {
                // The plain card: dark, a little lighter over its top half.
                card.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(52.0), ..default() },
                    BackgroundColor(CARD_TOP),
                    fade.with(CARD_TOP.alpha()),
                    FadeBack,
                ));
                // The title over the card.
                if let Some(title) = &m.title {
                    card.spawn((
                        Text::new(title.to_uppercase()),
                        font(&faces.bold, 19.0 * k),
                        // ("KILLED BY" in red.)
                        TextColor(Color::srgb(0.92, 0.22, 0.20)),
                        shadow(19.0 * k),
                        TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                        Node { position_type: PositionType::Absolute, left: Val::Px(0.0), bottom: Val::Px(h + 3.0 * k), ..default() },
                        fade,
                    ));
                }
                // His kill streak over the card: white on red, boxed.
                if let Some(streak) = &m.streak {
                    let red = Color::srgba(0.62, 0.08, 0.08, 0.86);
                    card.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            bottom: Val::Px(h + 4.0 * k),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(8.0 * k), Val::Px(2.0 * k)),
                                border: UiRect::all(Val::Px((1.0 * k).max(1.0))),
                                ..default()
                            },
                            BackgroundColor(red),
                            BorderColor::all(Color::srgba(0.95, 0.35, 0.30, 0.9)),
                            fade.with(red.alpha()),
                            FadeBack,
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(streak.to_uppercase()),
                                font(&faces.bold, 17.0 * k),
                                TextColor(Color::WHITE),
                                TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                                fade,
                            ));
                        });
                    });
                }
                // His calling card, covering the card, clipped to it.
                if let Some((image, pic)) = &m.card {
                    let s = crate::shell::pics::cover(*pic, Vec2::new(w, h));
                    card.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|clip| {
                        clip.spawn((
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
                        clip.spawn((
                            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                            BackgroundColor(CARD_SHADE),
                            fade.with(CARD_SHADE.alpha()),
                            FadeBack,
                        ));
                    });
                }
                // His emblem on the left.
                let side = (h - 14.0 * k).round();
                card.spawn((
                    Node {
                        width: Val::Px(side),
                        height: Val::Px(side),
                        margin: UiRect::left(Val::Px(7.0 * k)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    BackgroundColor(EMBLEM_BACK),
                    fade.with(EMBLEM_BACK.alpha()),
                    FadeBack,
                ))
                .with_children(|e| {
                    if let Some(icon) = &m.emblem {
                        e.spawn((ImageNode::new(icon.clone()), Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, fade));
                    }
                });
                // His name at the top.
                card.spawn((
                    Text::new(m.name.to_uppercase()),
                    font(&faces.bold, 22.0 * k),
                    TextColor(WHITE),
                    shadow(22.0 * k),
                    TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(side + 16.0 * k),
                        top: Val::Px(5.0 * k),
                        ..default()
                    },
                    fade,
                ));
                // His rank and level, small at the bottom right.
                card.spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(9.0 * k),
                    bottom: Val::Px(5.0 * k),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(3.0 * k),
                    ..default()
                })
                .with_children(|r| {
                    if let Some(icon) = &m.rank {
                        r.spawn((ImageNode::new(icon.clone()), Node { width: Val::Px(21.0 * k), height: Val::Px(21.0 * k), ..default() }, fade));
                    }
                    r.spawn((Text::new(m.level.to_string()), font(&faces.bold, 19.0 * k), TextColor(WHITE), shadow(19.0 * k), fade));
                });
            });
        })
        .id()
}

/// His rank icon on the card: Black Ops' for his level (no prestige),
/// else the combined progression's.
fn card_rank(
    level: i32,
    prestige: i32,
    cat: Option<&Catalog>,
    cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>,
    pics: Option<&mut crate::shell::pics::ShellPics>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
    if prestige <= 0
        && let Some(Some(name)) = cat.and_then(|c| c.bo1_ranks.get(bo1_rank_row(level)))
        && let Some((h, _)) = game_picture(cache, images, "t5", name)
    {
        return Some(h);
    }
    let name = bo2_profile::progression::rank_picture(level, prestige);
    pics?.get(&name, icons, images)
}

/// A calling card's or emblem's picture as a handle (in once read).
fn look_picture(cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>, images: &mut Assets<Image>, material: &str) -> Option<(Handle<Image>, Vec2)> {
    if let Some(Some(found)) = cache.get(material) {
        return Some(found.clone());
    }
    let img = crate::hud_cards::picture(material)?;
    let size = img.size().as_vec2();
    let mut img = (*img).clone();
    img.sampler = bevy::image::ImageSampler::linear();
    let found = (images.add(img), size);
    cache.insert(material.to_owned(), Some(found.clone()));
    Some(found)
}

/// A player's meta in the snapshot by name.
fn meta_named(snap: Option<&sim::Snapshot>, name: &str) -> Option<sim::ClientSnapshotMeta> {
    snap?.meta.clients.iter().find(|(_, m)| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)) == name).map(|(_, m)| m.clone())
}

/// A card's pictures for `model.name` (his look, his rank), and a key that
/// changes as they come in.
#[allow(clippy::too_many_arguments)]
fn dress_card(
    model: &mut BannerModel,
    me: &str,
    snap: Option<&sim::Snapshot>,
    slot: &crate::lui_hud::LuiHudSlot,
    cat: Option<&Catalog>,
    cache: &mut HashMap<String, Option<(Handle<Image>, Vec2)>>,
    pics: Option<&mut crate::shell::pics::ShellPics>,
    icons: Option<&assets::T6HudIcons>,
    images: &mut Assets<Image>,
) -> u32 {
    let mine = !me.is_empty() && model.name == me;
    let stats = mine.then(|| slot.his_look_stats());
    let (card, emblem) = crate::hud_cards::look_of(&model.name, stats.as_ref());
    model.card = look_picture(cache, images, &card);
    model.emblem = look_picture(cache, images, &emblem).map(|(h, _)| h);
    let meta = meta_named(snap, &model.name);
    let (level, prestige) = rank_of(&model.name, meta.as_ref(), mine, Some(slot));
    model.level = level;
    model.rank = card_rank(level, prestige, cat, cache, pics, icons, images);
    u32::from(model.rank.is_some()) | u32::from(model.card.is_some()) << 1 | u32::from(model.emblem.is_some()) << 2
}

#[derive(Default)]
struct BannerState {
    last_kill: Option<u32>,
    /// His kills since his last death (killstreak kills too).
    life_kills: u32,
    /// A kill waiting for its card: the victim and when.
    pending: Option<(String, f64)>,
    shown: Option<(BannerModel, u32)>,
}

/// While his Killed By card is up (real seconds it ends): no kill card then.
#[derive(Resource, Default)]
struct KilledByUntil(f64);

/// The card's slide and pop: it rises a little and settles as it comes in.
fn banner_pop(time: Res<Time<Real>>, window: Query<&Window, With<PrimaryWindow>>, mut q: Query<(&BannerRoot, &mut UiTransform)>) {
    let now = now_secs(&time);
    let k = window.single().map_or(1.0, |w| card_scale(Vec2::new(w.width(), w.height())));
    for (b, mut t) in &mut q {
        let age = (now - b.start) as f32;
        let (y, s) = card_motion(age);
        let (pos, size) = (Val2::px(0.0, y * k), Vec2::splat(s));
        if t.translation != pos || t.scale != size {
            t.translation = pos;
            t.scale = size;
        }
    }
}

/// The card's rise (px at 1080, down positive) and scale at `age`.
fn card_motion(age: f32) -> (f32, f32) {
    let t = (age / CARD_SETTLE).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - t).powi(3);
    (CARD_RISE * (1.0 - ease), 1.0 + CARD_POP * (1.0 - ease))
}

fn window_size(window: &Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    window.single().map_or(Vec2::new(1920.0, 1080.0), |w| Vec2::new(w.width(), w.height()))
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
    roots: Query<Entity, (With<BannerRoot>, Without<KilledByRoot>, Without<KillcamCardRoot>)>,
    killed_by: Res<KilledByUntil>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut sounds: MessageWriter<frame::UiPlaySound>,
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
        if !seen.me.is_empty() && k.victim == seen.me {
            st.life_kills = 0;
        }
        if seen.is_my_kill(k) {
            st.life_kills += 1;
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
            diag::info!(Ui, "bo2mp hud: no kill card for {victim}: the match ended on it");
        } else {
            diag::info!(Ui, "bo2mp hud: kill card: {victim}");
            if let Some(alias) = KILL_CARD_SOUND {
                sounds.write(frame::UiPlaySound { alias: alias.to_owned() });
            }
            let mut m = BannerModel::new(victim, 1, now);
            m.streak = Some(ordinal_kill(st.life_kills));
            st.shown = Some((m, u32::MAX));
        }
    }
    let life = (BANNER_IN + BANNER_HOLD + BANNER_OUT) as f64;
    // (His Killed By card has the place: he died, the kill card goes.)
    if st.shown.as_ref().is_some_and(|(m, _)| now - m.start >= life) || now < killed_by.0 {
        st.shown = None;
    }
    let Some((mut model, built)) = st.shown.clone() else {
        clear(&mut commands);
        return;
    };
    if !seen.hud {
        clear(&mut commands);
        st.shown = Some((model, u32::MAX));
        return;
    }
    let pictures = dress_card(&mut model, &seen.me, snap, &slot, cat.as_ref(), &mut cache, pics.as_deref_mut(), icons.as_deref(), &mut images);
    let size = window_size(&window);
    // Built again when a picture comes in or the window changes.
    let key = pictures | (size.y as u32) << 3 | (size.x as u32) << 16;
    if built == key && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    clear(&mut commands);
    spawn_banner(&mut commands, &faces, &model, size);
    st.shown = Some((model, key));
}

/// "1ST KILL", "2ND KILL", "11TH KILL", "23RD KILL".
fn ordinal_kill(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "TH",
        (1, _) => "ST",
        (2, _) => "ND",
        (3, _) => "RD",
        _ => "TH",
    };
    format!("{n}{suffix} KILL")
}

/// How long the Killed By card holds (BO2's obituary callout goes after
/// 2.5 s, or as the killcam starts).
const KILLED_BY_HOLD: f32 = 2.5 - BANNER_IN - BANNER_OUT;

#[derive(Default)]
struct KilledByState {
    /// The killer's client and the card as built (its key).
    shown: Option<(i32, BannerModel, u32)>,
    /// The death it was for (`bo2mp_killcam` "3|<killer>" seen) is over.
    done: bool,
}

/// The Killed By card at his death, before the killcam (BO2's own LUI one
/// is not sent under this HUD, `lui_frontend`): the killer's card under
/// "KILLED BY", where the kill card goes.
#[allow(clippy::too_many_arguments)]
fn killed_by_card(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (shell_fonts, game_font, mut pics): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>, Option<ResMut<crate::shell::pics::ShellPics>>),
    slot: NonSend<crate::lui_hud::LuiHudSlot>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<KilledByRoot>>,
    time: Res<Time<Real>>,
    mut until: ResMut<KilledByUntil>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<(Handle<Image>, Vec2)>>>,
    mut st: Local<KilledByState>,
) {
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let snap = presented.as_deref().and_then(PresentedSnapshot::snapshot);
    let found = match (snap, local.as_deref()) {
        (Some(snap), Some(l)) if crate::hud_modern::modern_hud_enabled() => {
            let dvars = snap.meta.script_dvars(l.0);
            let me = snap.meta.for_client(l.0).map(|m| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned()).unwrap_or_default();
            let killer = dvars
                .string("bo2mp_killcam")
                .and_then(|k| k.strip_prefix("3|"))
                .and_then(|k| k.trim().parse::<i32>().ok())
                .filter(|k| *k >= 0)
                .filter(|_| dvars.string("bo2zm_hud_hidden") != Some("1"));
            killer.map(|k| (k, me))
        }
        _ => None,
    };
    let now = now_secs(&time);
    let Some((killer, me)) = found else {
        clear(&mut commands);
        *st = KilledByState::default();
        until.0 = 0.0;
        return;
    };
    if st.done {
        clear(&mut commands);
        return;
    }
    let mut model = match st.shown.take() {
        Some((k, m, built)) if k == killer => {
            st.shown = Some((k, m.clone(), built));
            m
        }
        _ => {
            let name = snap
                .and_then(|s| s.meta.clients.iter().find(|(c, _)| c.0 as i32 == killer))
                .map(|(_, m)| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned())
                .unwrap_or_default();
            if name.is_empty() {
                clear(&mut commands);
                return;
            }
            diag::info!(Ui, "bo2mp hud: killed-by card: {name} (client {killer})");
            let mut m = BannerModel::new(name, 1, now);
            m.title = Some("Killed By".into());
            m.hold = KILLED_BY_HOLD;
            st.shown = Some((killer, m.clone(), u32::MAX));
            m
        }
    };
    let life = (BANNER_IN + model.hold + BANNER_OUT) as f64;
    if now - model.start >= life {
        st.done = true;
        until.0 = 0.0;
        clear(&mut commands);
        return;
    }
    until.0 = model.start + life;
    let cat = catalog();
    let pictures = dress_card(&mut model, &me, snap, &slot, cat.as_ref(), &mut cache, pics.as_deref_mut(), icons.as_deref(), &mut images);
    let size = window_size(&window);
    let key = pictures | (size.y as u32) << 3 | (size.x as u32) << 16;
    if st.shown.as_ref().is_some_and(|(_, _, b)| *b == key) && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    clear(&mut commands);
    let e = spawn_banner(&mut commands, &faces, &model, size);
    commands.entity(e).insert(KilledByRoot);
    st.shown = Some((killer, model, key));
}

/// The killcam's card (a `BannerRoot` too, for its slide in).
#[derive(Component)]
struct KillcamCardRoot;

/// Where the killcam card's middle sits, as a share of the screen's height:
/// where BO2's Killcam widget put its NamePlate (killcam.lua: the bottom
/// band, 120 of 720 high, the plate anchored to its bottom from -2 to -1
/// `PlayerCallout_Height` (60), so 600..660), just above the respawn and
/// skip prompts under it.
const KILLCAM_CARD_CENTRE: f32 = 630.0 / 720.0;

/// It holds while the killcam plays (it goes with it, no fade out).
const KILLCAM_CARD_HOLD: f32 = 3600.0;

/// The killer's card through the killcam and the final killcam
/// (`bo2mp_killcam` "1|<killer>" / "2|<killer>"), in place of the NamePlate
/// BO2's LUI Killcam widget draws (that one is not made under this HUD,
/// `lui_frontend::modern_killcam_card`; the widget's KILLCAM title, its
/// bands and the prompts stay BO2's). No title: the widget's says it.
#[allow(clippy::too_many_arguments)]
fn killcam_card(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (shell_fonts, game_font, mut pics): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>, Option<ResMut<crate::shell::pics::ShellPics>>),
    slot: NonSend<crate::lui_hud::LuiHudSlot>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<KillcamCardRoot>>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<(Handle<Image>, Vec2)>>>,
    // The killer's client and the card as built (its key).
    mut st: Local<Option<(i32, BannerModel, u32)>>,
) {
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    let snap = presented.as_deref().and_then(PresentedSnapshot::snapshot);
    let found = match (snap, local.as_deref()) {
        (Some(snap), Some(l)) if crate::hud_modern::modern_hud_enabled() => {
            let dvars = snap.meta.script_dvars(l.0);
            let me = snap.meta.for_client(l.0).map(|m| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned()).unwrap_or_default();
            // (Not "3|": that is the Killed By card at his death.)
            let killer = dvars
                .string("bo2mp_killcam")
                .and_then(|k| k.strip_prefix("1|").or_else(|| k.strip_prefix("2|")))
                .and_then(|k| k.trim().parse::<i32>().ok())
                .filter(|k| *k >= 0);
            killer.map(|k| (k, me))
        }
        _ => None,
    };
    let Some((killer, me)) = found else {
        clear(&mut commands);
        *st = None;
        return;
    };
    let now = now_secs(&time);
    let mut model = match st.take() {
        Some((k, m, built)) if k == killer => {
            *st = Some((k, m.clone(), built));
            m
        }
        _ => {
            let name = snap
                .and_then(|s| s.meta.clients.iter().find(|(c, _)| c.0 as i32 == killer))
                .map(|(_, m)| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned())
                .unwrap_or_default();
            if name.is_empty() {
                clear(&mut commands);
                return;
            }
            diag::info!(Ui, "bo2mp hud: killcam card: {name} (client {killer})");
            let mut m = BannerModel::new(name, 1, now);
            m.hold = KILLCAM_CARD_HOLD;
            *st = Some((killer, m.clone(), u32::MAX));
            m
        }
    };
    let cat = catalog();
    let pictures = dress_card(&mut model, &me, snap, &slot, cat.as_ref(), &mut cache, pics.as_deref_mut(), icons.as_deref(), &mut images);
    let size = window_size(&window);
    let key = pictures | (size.y as u32) << 3 | (size.x as u32) << 16;
    if st.as_ref().is_some_and(|(_, _, b)| *b == key) && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    clear(&mut commands);
    let e = spawn_banner(&mut commands, &faces, &model, size);
    // (Lower than the kill card's place: where BO2's plate was.)
    let h = (CARD_H * card_scale(size)).round();
    commands.entity(e).insert((KillcamCardRoot, Node {
        position_type: PositionType::Absolute,
        top: Val::Px((size.y * KILLCAM_CARD_CENTRE - h * 0.5).round()),
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    }));
    *st = Some((killer, model, key));
}

// --- Scorestreak callouts ------------------------------------------------------

/// BO2's `player_callout` notifies ("<player> called in a UAV"): with the
/// modern HUD they come here (`lui_frontend`) instead of BO2's LUI popup,
/// as (message key or text, client), and show as our card: the player's
/// MW2/MW3 calling card and emblem with the message over it.
static CALLOUTS: std::sync::Mutex<Vec<(String, i32)>> = std::sync::Mutex::new(Vec::new());

pub(crate) fn push_callout(message: &str, client: i32) {
    if let Ok(mut q) = CALLOUTS.lock() {
        q.push((message.to_owned(), client));
        let n = q.len();
        q.drain(..n.saturating_sub(4));
    }
}

#[derive(Component)]
struct CalloutRoot;

#[derive(Default)]
struct CalloutState {
    shown: Option<(BannerModel, f64)>,
    waiting: std::collections::VecDeque<(String, i32)>,
}

/// A callout's message as shown: its localized text (`&KILLSTREAK_..`
/// keys), its player's name put in for `&&1`, upper case.
fn callout_text(raw: &str, name: &str, slot: &crate::lui_hud::LuiHudSlot) -> String {
    let key = raw.trim().trim_start_matches('&');
    let text = slot.localize(key).unwrap_or_else(|| key.replace('_', " "));
    text.replace("&&1", name).to_uppercase()
}

#[allow(clippy::too_many_arguments)]
fn streak_callout(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    icons: Option<Res<assets::T6HudIcons>>,
    (shell_fonts, game_font, mut pics): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>, Option<ResMut<crate::shell::pics::ShellPics>>),
    slot: NonSend<crate::lui_hud::LuiHudSlot>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<CalloutRoot>>,
    time: Res<Time<Real>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<String, Option<(Handle<Image>, Vec2)>>>,
    mut st: Local<CalloutState>,
) {
    if let Ok(mut q) = CALLOUTS.lock() {
        st.waiting.extend(q.drain(..));
    }
    let now = now_secs(&time);
    let clear = |commands: &mut Commands| {
        for e in &roots {
            commands.entity(e).despawn();
        }
    };
    if let Some((m, _)) = &st.shown
        && now - m.start < (BANNER_IN + m.hold + BANNER_OUT) as f64
    {
        return;
    }
    clear(&mut commands);
    st.shown = None;
    let (Some(p), Some(l)) = (presented.as_deref(), local.as_deref()) else { return };
    let Some(snap) = p.snapshot() else { return };
    let Some((raw, client)) = st.waiting.pop_front() else { return };
    let name = snap
        .meta
        .clients
        .iter()
        .find(|(c, _)| c.0 as i32 == client)
        .map(|(_, m)| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned())
        .unwrap_or_default();
    if name.is_empty() {
        return;
    }
    let me = snap.meta.for_client(l.0).map(|m| String::from_utf8_lossy(&m.name).trim_end_matches(char::from(0)).to_owned()).unwrap_or_default();
    let mut model = BannerModel::new(name.clone(), 1, now);
    model.title = Some(callout_text(&raw, &name, &slot));
    model.hold = 2.5;
    let cat = catalog();
    dress_card(&mut model, &me, Some(snap), &slot, cat.as_ref(), &mut cache, pics.as_deref_mut(), icons.as_deref(), &mut images);
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
    let size = window_size(&window);
    let e = spawn_banner(&mut commands, &faces, &model, size);
    // (Above the kill card's place: upper middle, under the score bar.)
    commands.entity(e).insert((CalloutRoot, Node {
        position_type: PositionType::Absolute,
        top: Val::Percent(14.0),
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        ..default()
    }));
    diag::info!(Ui, "bo2mp hud: scorestreak callout: {name}: {raw}");
    st.shown = Some((model, now));
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
    app.init_resource::<KilledByUntil>();
    app.add_systems(
        Update,
        (kill_feed, hit_marker, killed_by_card, killcam_card, kill_banner.after(killed_by_card), scoreboard, streak_callout).in_set(ClientSet::Ui),
    )
    .add_systems(Update, (banner_pop, apply_fades).in_set(ClientSet::Ui).after(kill_banner).after(killed_by_card).after(killcam_card).after(kill_feed));
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
                    4 => {
                        // (Black Ops' private: one broad green chevron.)
                        let d = (u - 0.5).abs();
                        let on = d < 0.42 && v > 0.30 + d * 0.62 && v < 0.62 + d * 0.62;
                        if on { [0.42 + 0.2 * (1.0 - v), 0.62 + 0.15 * (1.0 - v), 0.10, 1.0] } else { [0.0; 4] }
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
        window: Query<&Window, With<PrimaryWindow>>,
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
            let path = plan.out.join(format!("{}_{}p.png", names[i], super::window_size(&window).y as u32));
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
        let size = super::window_size(&window);
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
                let e = spawn_feed(&mut commands, &faces, &rows, size.y);
                mark(&mut commands, e);
                // (Black Ops' rank icon when its IWDs are under IW4L_GAMES,
                // else a green chevron stand-in; the Modern Warfare calling
                // card and emblem when their games are there, else stand-ins
                // (case 1: none, the plain drawn card).)
                let mut cache = HashMap::new();
                let bo1_rank = game_picture(&mut cache, &mut images, "t5", "rank_pvt1_128").map(|(h, _)| h);
                println!("hud preview: BO1 rank_pvt1_128 {}", if bo1_rank.is_some() { "read" } else { "not read (stand-in)" });
                let mut model = BannerModel::new("Grim_Gorilla".into(), 1, now);
                model.rank = bo1_rank.or_else(|| Some(images.add(stand_in(4, 128, 128))));
                if i == 0 {
                    model.card = Some((images.add(stand_in(0, 512, 128)), Vec2::new(512.0, 128.0)));
                    model.emblem = Some(images.add(stand_in(3, 64, 64)));
                } else {
                    model.title = Some("Killed By".into());
                }
                let e = spawn_banner(&mut commands, &faces, &model, size);
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
    fn black_ops_rank_rows_by_level() {
        assert_eq!(bo1_rank_row(0), 0);
        assert_eq!(bo1_rank_row(1), 0);
        assert_eq!(bo1_rank_row(3), 0);
        assert_eq!(bo1_rank_row(4), 1);
        assert_eq!(BO1_RANKS[bo1_rank_row(49)].1[0], "rank_gen1_128");
        assert_eq!(BO1_RANKS[bo1_rank_row(50)].1[0], "rank_comm_128");
        // BO2's levels past Black Ops' 50 keep its last.
        assert_eq!(bo1_rank_row(55), BO1_RANKS.len() - 1);
        // The card rises and settles, then stays put.
        let (y0, s0) = card_motion(0.0);
        assert!(y0 > 20.0 && s0 > 1.0);
        assert_eq!(card_motion(CARD_SETTLE), (0.0, 1.0));
        const { assert!(KILL_CARD_SOUND.is_none()) };
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
