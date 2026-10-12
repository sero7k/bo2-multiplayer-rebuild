//! bo2mp: medals as Black Ops drew them, in place of BO2's medal popup
//! while the modern HUD is on (`hud_modern::modern_hud_enabled`).
//!
//! BO2's scripts still decide every medal: `_medals::codecallback_medal`
//! sends `luinotifyevent(&"medal_received", 1, <scoreInfo row>)`, which
//! reaches the client in `bo2mp_lui`. `lui_frontend::match_events` keeps
//! that notify from BO2's `NotificationPopups` (no medal art, no BO2 medal
//! popup); this module reads the same notify and shows Black Ops' compact
//! notification under the top of the screen:
//!
//! - the medal's title, large and bold, in white ("One Shot, One Kill");
//! - its description underneath, smaller ("Killed an enemy with one
//!   shot."), followed by its points in Black Ops' yellow (`MEDAL_XP_POPUP`,
//!   `^3+&&1^7`);
//! - on a dark translucent box with a thin orange accent.
//!
//! The text is Black Ops' own (`MEDAL_*` / `MEDAL_*_DESC` from its English
//! zones, read once off the main thread) where BO1 had that medal, else
//! BO2's (`MEDAL_<x>` / `MEDAL_<x>_DESC`, the scoreInfo row's Medal
//! Reference). The points are the row's XP for the game type in BO2's
//! `mp/scoreInfo.csv` (`<gametype> XP`; its `<gametype> score` when it has
//! no XP): the same table the server scores with. Nothing here scores.
//!
//! Medals that come close together queue: each shows ~1.8 s (fade and
//! slide in, hold, fade and rise out), quicker while more wait.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::hud_match::Faces;
use crate::layers::{GameUiFont, UiLayer, UiLayerVisibility};
use crate::shell::theme::ShellFonts;

/// BO2's table the medal rows index.
const SCORE_INFO: &str = "mp/scoreInfo.csv";

/// A medal's fade in, its whole life, its fade out (s). While two or more
/// wait, each is shown for `QUICK_LIFE` instead.
const MEDAL_IN: f32 = 0.18;
const MEDAL_LIFE: f32 = 1.8;
const QUICK_LIFE: f32 = 1.3;
const MEDAL_OUT: f32 = 0.32;
/// The most medals that wait; more are dropped (logged).
const MAX_WAITING: usize = 8;
/// Where the box sits: its top, as a share of the screen height (BO2's
/// medal sat a little higher, ~20%).
const TOP: f32 = 0.255;

/// Black Ops' look: white title, light description, yellow points
/// (its `^3`), a dark box and an orange accent.
const TITLE: Color = Color::srgb(1.0, 1.0, 1.0);
const DESC: Color = Color::srgba(0.90, 0.90, 0.88, 0.95);
const POINTS: Color = Color::srgb(1.0, 0.80, 0.22);
const BOX: Color = Color::srgba(0.01, 0.01, 0.012, 0.66);
const ACCENT: Color = Color::srgb(1.0, 0.58, 0.12);

// --- Medal text and points ----------------------------------------------------------

/// A BO2 medal event (scoreInfo's Script Reference) and the Black Ops
/// text it shows: its title key, and its description key when it is not
/// `<title>_DESC` (`None` in the second place: BO2's description, as Black
/// Ops' says something else).
const BO1_TEXT: &[(&str, &str, Option<&str>)] = &[
    (
        "kill_enemy_one_bullet",
        "MEDAL_ONE_SHOT_KILL",
        Some("MEDAL_ONE_SHOT_KILL_DESC"),
    ),
    ("headshot", "MEDAL_HEAD_SHOT", Some("MEDAL_HEAD_SHOT_DESC")),
    ("revenge_kill", "MEDAL_PAYBACK", Some("MEDAL_PAYBACK_DESC")),
    (
        "first_kill",
        "MEDAL_FIRST_BLOOD",
        Some("MEDAL_FIRST_BLOOD_DESC"),
    ),
    (
        "kill_enemy_after_death",
        "MEDAL_AFTERLIFE",
        Some("MEDAL_AFTERLIFE_DESC"),
    ),
    (
        "assisted_suicide",
        "MEDAL_ASSISTED_SUICIDE",
        Some("MEDAL_ASSISTED_SUICIDE_DESC"),
    ),
    (
        "kill_enemy_who_killed_teammate",
        "MEDAL_AVENGER",
        Some("MEDAL_AVENGER_DESC"),
    ),
    (
        "kill_enemy_injuring_teammate",
        "MEDAL_RESCUER",
        Some("MEDAL_RESCUER_DESC"),
    ),
    // (BO1's was a knife in the back; BO2's is any kill from behind.)
    ("backstabber_kill", "MEDAL_BACK_STABBER", None),
    (
        "ballistic_knife_kill",
        "MEDAL_BALLISTIC_KNIFE_KILL",
        Some("MEDAL_BALLISTIC_KNIFE_KILL_DESC"),
    ),
    (
        "bounce_hatchet_kill",
        "MEDAL_BANK_SHOT",
        Some("MEDAL_BANK_SHOT_DESC"),
    ),
    (
        "hatchet_kill",
        "MEDAL_BULLS_EYE",
        Some("MEDAL_BULLS_EYE_DESC"),
    ),
    (
        "stop_enemy_killstreak",
        "MEDAL_BUZZ_KILL",
        Some("MEDAL_BUZZ_KILL_DESC"),
    ),
    (
        "comeback_from_deathstreak",
        "MEDAL_COMEBACK",
        Some("MEDAL_COMEBACK_DESC"),
    ),
    (
        "longshot_kill",
        "MEDAL_LONG_SHOT",
        Some("MEDAL_LONG_SHOT_DESC"),
    ),
    (
        "capture_enemy_crate",
        "MEDAL_HIJACKER",
        Some("MEDAL_HIJACKER_DESC"),
    ),
    (
        "stick_explosive_kill",
        "MEDAL_STUCK_TO_PLAYER",
        Some("MEDAL_STUCK_TO_PLAYER_DESC"),
    ),
    (
        "multikill_2",
        "MEDAL_DOUBLE_KILL",
        Some("MEDAL_DOUBLE_KILL_DESC"),
    ),
    (
        "multikill_3",
        "MEDAL_TRIPLE_KILL",
        Some("MEDAL_TRIPLE_KILL_DESC"),
    ),
    (
        "multikill_4",
        "MEDAL_MULTI_KILL",
        Some("MEDAL_MULTI_KILL_DESC"),
    ),
    (
        "multikill_5",
        "MEDAL_MULTI_KILL",
        Some("MEDAL_MULTI_KILL_DESC"),
    ),
    (
        "multikill_6",
        "MEDAL_MULTI_KILL",
        Some("MEDAL_MULTI_KILL_DESC"),
    ),
    (
        "multikill_7",
        "MEDAL_MULTI_KILL",
        Some("MEDAL_MULTI_KILL_DESC"),
    ),
    (
        "multikill_8",
        "MEDAL_MULTI_KILL",
        Some("MEDAL_MULTI_KILL_DESC"),
    ),
    (
        "multikill_more_than_8",
        "MEDAL_MULTI_KILL",
        Some("MEDAL_MULTI_KILL_DESC"),
    ),
    ("bomb_detonated", "MEDAL_BOMBER", Some("MEDAL_BOMBER_DESC")),
    ("defused_bomb", "MEDAL_HERO", Some("MEDAL_HERO_DESC")),
    (
        "killed_bomb_defuser",
        "MEDAL_ANTI_BOMBER",
        Some("MEDAL_ANTI_BOMBER_DESC"),
    ),
    (
        "flag_capture",
        "MEDAL_FLAG_CAPTURE",
        Some("MEDAL_FLAG_CAPTURE_DESC"),
    ),
    (
        "dom_point_secured",
        "MEDAL_POSITION_SECURE",
        Some("MEDAL_POSITION_SECURE_DESC"),
    ),
    (
        "hq_secure",
        "MEDAL_POSITION_SECURE",
        Some("MEDAL_POSITION_SECURE_DESC"),
    ),
    (
        "koth_secure",
        "MEDAL_POSITION_SECURE",
        Some("MEDAL_POSITION_SECURE_DESC"),
    ),
    (
        "destroyed_rcbomb",
        "MEDAL_DESTROYER_RCBOMB",
        Some("MEDAL_DESTROYER_RCBOMB_DESC"),
    ),
    (
        "destroyed_sentry_gun",
        "MEDAL_DESTROYER_TURRET",
        Some("MEDAL_DESTROYER_TURRET_DESC"),
    ),
    (
        "destroyed_microwave_turret",
        "MEDAL_DESTROYER_TURRET",
        Some("MEDAL_DESTROYER_TURRET_DESC"),
    ),
    (
        "destroyed_helicopter_comlink",
        "MEDAL_DESTROYER",
        Some("MEDAL_DESTROYER_HELICOPTER_DESC"),
    ),
    (
        "destroyed_helicopter_guard",
        "MEDAL_DESTROYER",
        Some("MEDAL_DESTROYER_HELICOPTER_DESC"),
    ),
    (
        "destroyed_helicopter_gunner",
        "MEDAL_DESTROYER",
        Some("MEDAL_DESTROYER_HELICOPTER_DESC"),
    ),
    (
        "destroyed_helicopter_supply_drop",
        "MEDAL_DESTROYER",
        Some("MEDAL_DESTROYER_HELICOPTER_DESC"),
    ),
];

/// A medal's row in `mp/scoreInfo.csv` as this game type reads it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MedalRow {
    /// Script Reference (`kill_enemy_one_bullet`).
    pub event: String,
    /// Medal Reference (`MEDAL_KILL_ENEMY_ONE_BULLET`).
    pub name_key: String,
    /// `<gametype> XP` and `<gametype> score`.
    pub xp: i32,
    pub score: i32,
}

/// The medal in `row` (header first, as the table's own rows count), its
/// XP and score for `gametype` (TDM's columns when the game type has none).
pub(crate) fn medal_row(header: &[String], row: &[String], gametype: &str) -> Option<MedalRow> {
    let cell = |i: usize| row.get(i).map_or("", |c| c.trim());
    let name_key = cell(2).to_owned();
    if name_key.is_empty() {
        return None;
    }
    let col = |what: &str| {
        let find = |g: &str| {
            header
                .iter()
                .position(|h| h.trim().eq_ignore_ascii_case(&format!("{g} {what}")))
        };
        find(gametype).or_else(|| find("tdm"))
    };
    let num = |c: Option<usize>| {
        c.and_then(|i| cell(i).parse::<f32>().ok())
            .map_or(0, |v| v as i32)
    };
    Some(MedalRow {
        event: cell(0).to_owned(),
        name_key,
        xp: num(col("XP")),
        score: num(col("score")),
    })
}

impl MedalRow {
    /// The points Black Ops' popup shows: the XP the medal gives (its
    /// `MEDAL_XP_POPUP`), else its score; none when it gives neither.
    pub(crate) fn points(&self) -> Option<i32> {
        [self.xp, self.score].into_iter().find(|p| *p > 0)
    }
}

/// Where a medal's text came from (for the log).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextSource {
    Bo1,
    Bo2,
    /// Neither game's text: the event's name.
    Name,
}

/// A medal as drawn.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MedalModel {
    pub title: String,
    pub desc: String,
    /// "+100", or empty.
    pub points: String,
}

/// Text without the games' colour codes (`^3`).
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(|n| n.is_ascii_digit()) {
            chars.next();
            continue;
        }
        out.push(c);
    }
    out.trim().to_owned()
}

/// `kill_enemy_one_bullet` -> `Kill Enemy One Bullet`.
fn title_of(event: &str) -> String {
    event
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The medal's title and description (Black Ops' where it had the medal,
/// else BO2's) and its points as Black Ops wrote them (`MEDAL_XP_POPUP`,
/// else "+N").
pub(crate) fn medal_model(
    row: &MedalRow,
    bo1: &dyn Fn(&str) -> Option<String>,
    bo2: &dyn Fn(&str) -> Option<String>,
) -> (MedalModel, TextSource) {
    let text = |f: &dyn Fn(&str) -> Option<String>, k: &str| {
        f(k).map(|t| plain(&t)).filter(|t| !t.is_empty())
    };
    let from_bo1 = BO1_TEXT
        .iter()
        .find(|(e, _, _)| e.eq_ignore_ascii_case(&row.event))
        .and_then(|(_, title, desc)| {
            let t = text(bo1, title)?;
            let d = match desc {
                Some(d) => text(bo1, d),
                None => text(bo2, &format!("{}_DESC", row.name_key)),
            };
            Some((t, d.unwrap_or_default()))
        });
    let (title, desc, source) = match from_bo1 {
        Some((t, d)) => (t, d, TextSource::Bo1),
        None => match text(bo2, &row.name_key) {
            Some(t) => (
                t,
                text(bo2, &format!("{}_DESC", row.name_key)).unwrap_or_default(),
                TextSource::Bo2,
            ),
            None => (title_of(&row.event), String::new(), TextSource::Name),
        },
    };
    let points = row.points().map_or_else(String::new, |p| {
        bo1("MEDAL_XP_POPUP")
            .filter(|f| f.contains("&&1"))
            .map(|f| plain(&f.replace("&&1", &p.to_string())))
            .unwrap_or_else(|| format!("+{p}"))
    });
    (
        MedalModel {
            title,
            desc,
            points,
        },
        source,
    )
}

// --- Black Ops' text ---------------------------------------------------------------------

/// Black Ops' `MEDAL_*` strings from its English zones (`<lang>_patch_mp`,
/// then `<lang>_common_mp`), read once off the main thread: empty until
/// then, and for good when Black Ops is not installed.
fn bo1_strings() -> Option<std::sync::Arc<HashMap<String, String>>> {
    static FOUND: OnceLock<Mutex<Option<std::sync::Arc<HashMap<String, String>>>>> =
        OnceLock::new();
    let slot = FOUND.get_or_init(|| {
        std::thread::Builder::new()
            .name("hud-medal-text".into())
            .spawn(|| {
                let found = std::sync::Arc::new(read_bo1_strings());
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

fn read_bo1_strings() -> HashMap<String, String> {
    let mut out = HashMap::new();
    let found = asset_transport::games_root_from_env()
        .and_then(|root| asset_transport::find_zone_file(&root, "t5:common_mp"))
        .map(|common| common.path);
    let common = match found {
        Ok(p) => p,
        Err(e) => {
            diag::info!(
                Ui,
                "bo2mp hud: medals: no Black Ops zones ({e}): BO2's medal text"
            );
            return out;
        }
    };
    let zone = |stem: &str| {
        asset_transport::discover::find_t5_localized_zone(&common, Some("english"), stem)
            .or_else(|_| asset_transport::discover::find_t5_localized_zone(&common, None, stem))
            .ok()
            .flatten()
    };
    for stem in ["patch_mp", "common_mp"] {
        let Some(z) = zone(stem) else { continue };
        match asset_game::load_localize_catalog_t5(&z.path) {
            Ok(cat) => {
                for (k, v) in cat.iter().filter(|(k, _)| k.starts_with("MEDAL_")) {
                    out.entry(k.to_owned()).or_insert_with(|| v.to_owned());
                }
            }
            Err(e) => diag::warn!(Ui, "bo2mp hud: medals: {}: {e}", z.path.display()),
        }
    }
    diag::info!(
        Ui,
        "bo2mp hud: medals: {} Black Ops medal strings",
        out.len()
    );
    out
}

// --- The queue ------------------------------------------------------------------------------

/// Medals waiting and the one shown (since when, for how long).
#[derive(Default)]
pub(crate) struct MedalQueue {
    waiting: VecDeque<MedalModel>,
    shown: Option<(MedalModel, f64, f32)>,
    /// How many have started (each a new box).
    pub started: u64,
}

impl MedalQueue {
    /// A medal to show after those already waiting (false: too many wait).
    pub(crate) fn push(&mut self, m: MedalModel) -> bool {
        if self.waiting.len() >= MAX_WAITING {
            return false;
        }
        self.waiting.push_back(m);
        true
    }

    /// The medal shown at `now` (its start and life): the one shown until
    /// its life is over, then the next one waiting, from `now`.
    pub(crate) fn tick(&mut self, now: f64) -> Option<(&MedalModel, f64, f32)> {
        if self
            .shown
            .as_ref()
            .is_some_and(|(_, start, life)| now - start >= f64::from(*life))
        {
            self.shown = None;
        }
        if self.shown.is_none()
            && let Some(next) = self.waiting.pop_front()
        {
            // (Quicker while others wait, so a run of medals keeps up.)
            let life = if self.waiting.len() >= 2 {
                QUICK_LIFE
            } else {
                MEDAL_LIFE
            };
            self.shown = Some((next, now, life));
            self.started += 1;
        }
        self.shown.as_ref().map(|(m, s, l)| (m, *s, *l))
    }

    pub(crate) fn waiting(&self) -> usize {
        self.waiting.len()
    }
}

/// One `bo2mp_lui` notify: its number, its event, its arguments.
fn parse_lui(feed: &str) -> Vec<(u32, &str, Vec<&str>)> {
    feed.split(';')
        .filter_map(|item| {
            let mut parts = item.split('|');
            let n = parts.next()?.trim().parse::<u32>().ok()?;
            let name = parts.next()?;
            Some((n, name, parts.collect()))
        })
        .collect()
}

/// The new `medal_received` rows since `last` (each notify once). The first
/// look takes what is there as old; a feed all below `last` is a new
/// match's (its count starts again).
fn new_medals(feed: &str, last: &mut Option<u32>) -> Vec<usize> {
    let items = parse_lui(feed);
    let top = items.iter().map(|(n, _, _)| *n).max();
    let Some(seen) = *last else {
        *last = Some(top.unwrap_or(0));
        return Vec::new();
    };
    let from = if top.is_some_and(|t| t < seen) {
        0
    } else {
        seen
    };
    let mut out: Vec<(u32, usize)> = items
        .iter()
        .filter(|(n, name, _)| *n > from && *name == "medal_received")
        .filter_map(|(n, _, args)| Some((*n, args.first()?.trim().parse::<f32>().ok()? as usize)))
        .collect();
    out.sort_by_key(|(n, _)| *n);
    *last = Some(top.map_or(from, |t| t.max(from)));
    out.into_iter().map(|(_, row)| row).collect()
}

// --- Drawing -----------------------------------------------------------------------------------

/// The box (its start and life on the real clock).
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct MedalRoot {
    pub start: f64,
    pub life: f32,
}

/// A piece of the box that fades with it: its full colour, and whether
/// the colour is its background (else its text).
#[derive(Component, Clone, Copy, Debug)]
struct MedalPiece {
    color: Color,
    back: bool,
    shadow: f32,
}

/// The box's look at `age` into a `life`: its opacity, its rise (px at
/// 1080, down positive) and its scale (a little bigger as it lands).
pub(crate) fn medal_motion(age: f32, life: f32) -> (f32, f32, f32) {
    let fin = (age / MEDAL_IN).clamp(0.0, 1.0);
    let fout = ((life - age) / MEDAL_OUT).clamp(0.0, 1.0);
    let ease = |t: f32| 1.0 - (1.0 - t) * (1.0 - t);
    let y = -16.0 * (1.0 - ease(fin)) - 10.0 * (1.0 - fout);
    let scale = 1.0 + 0.10 * (1.0 - ease(fin));
    (fin.min(fout), y, scale)
}

fn font(h: &Handle<Font>, px: f32) -> TextFont {
    crate::shell::theme::font(h, px)
}

fn shadow(px: f32) -> TextShadow {
    TextShadow {
        offset: Vec2::splat((px / 16.0).max(1.0)),
        color: Color::srgba(0.0, 0.0, 0.0, 0.8),
        ..default()
    }
}

/// Black Ops' medal box: the title over the description and its points.
pub(crate) fn spawn_medal(
    commands: &mut Commands,
    faces: &Faces,
    m: &MedalModel,
    height: f32,
    root: MedalRoot,
) -> Entity {
    let k = (height / 1080.0).max(0.4);
    let text = |c: Color| MedalPiece {
        color: c,
        back: false,
        shadow: 0.8,
    };
    let back = |c: Color| MedalPiece {
        color: c,
        back: true,
        shadow: 0.0,
    };
    commands
        .spawn((
            root,
            UiLayer::Hud,
            UiLayerVisibility,
            GlobalZIndex(112),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(TOP * 100.0),
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
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::new(
                        Val::Px(30.0 * k),
                        Val::Px(30.0 * k),
                        Val::Px(8.0 * k),
                        Val::Px(9.0 * k),
                    ),
                    min_width: Val::Px(300.0 * k),
                    ..default()
                },
                BackgroundColor(BOX),
                back(BOX),
            ))
            .with_children(|b| {
                // The orange line along the top, brightest at the middle.
                b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Px((2.0 * k).max(1.0)),
                        ..default()
                    },
                    accent_line(1.0),
                    MedalAccent::Line,
                ));
                // A faint warm glow under it.
                b.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(60.0),
                        ..default()
                    },
                    accent_glow(1.0),
                    MedalAccent::Glow,
                ));
                b.spawn((
                    Text::new(m.title.clone()),
                    font(&faces.bold, 36.0 * k),
                    TextColor(TITLE),
                    shadow(36.0 * k),
                    text(TITLE),
                ));
                b.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Baseline,
                    column_gap: Val::Px(7.0 * k),
                    ..default()
                })
                .with_children(|r| {
                    if !m.desc.is_empty() {
                        r.spawn((
                            Text::new(m.desc.clone()),
                            font(&faces.regular, 20.0 * k),
                            TextColor(DESC),
                            shadow(20.0 * k),
                            text(DESC),
                        ));
                    }
                    if !m.points.is_empty() {
                        r.spawn((
                            Text::new(m.points.clone()),
                            font(&faces.bold, 21.0 * k),
                            TextColor(POINTS),
                            shadow(21.0 * k),
                            text(POINTS),
                        ));
                    }
                });
            });
        })
        .id()
}

/// The accent's gradients (redrawn as the box fades).
#[derive(Component, Clone, Copy)]
enum MedalAccent {
    Line,
    Glow,
}

impl MedalAccent {
    fn at(self, a: f32) -> BackgroundGradient {
        match self {
            MedalAccent::Line => accent_line(a),
            MedalAccent::Glow => accent_glow(a),
        }
    }
}

/// A faint warm wash from the top line down (blended in linear light, so
/// it stays faint).
fn accent_glow(a: f32) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::new(ACCENT.with_alpha(0.035 * a), Val::Percent(0.0)),
        ColorStop::new(ACCENT.with_alpha(0.0), Val::Percent(100.0)),
    ]))
}

fn accent_line(a: f32) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::to_right(vec![
        ColorStop::new(ACCENT.with_alpha(0.0), Val::Percent(0.0)),
        ColorStop::new(ACCENT.with_alpha(0.9 * a), Val::Percent(50.0)),
        ColorStop::new(ACCENT.with_alpha(0.0), Val::Percent(100.0)),
    ]))
}

/// Each box's fade, rise and scale for now.
#[allow(clippy::type_complexity)]
fn medal_animate(
    time: Res<Time<Real>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(Entity, &MedalRoot, &mut UiTransform)>,
    children: Query<&Children>,
    mut pieces: Query<(
        Option<&MedalPiece>,
        Option<&mut TextColor>,
        Option<&mut BackgroundColor>,
        Option<&mut TextShadow>,
        Option<(&MedalAccent, &mut BackgroundGradient)>,
    )>,
) {
    let now = time.elapsed_secs_f64();
    let k = window
        .single()
        .map_or(1.0, |w| (w.height() / 1080.0).max(0.4));
    for (e, root, mut t) in &mut roots {
        let (a, y, s) = medal_motion((now - root.start) as f32, root.life);
        let (pos, size) = (Val2::px(0.0, y * k), Vec2::splat(s));
        if t.translation != pos || t.scale != size {
            t.translation = pos;
            t.scale = size;
        }
        for child in children.iter_descendants(e) {
            let Ok((piece, text, back, shade, grad)) = pieces.get_mut(child) else {
                continue;
            };
            if let Some((kind, mut g)) = grad {
                *g = kind.at(a);
            }
            let Some(p) = piece else { continue };
            let c = p.color.with_alpha(p.color.alpha() * a);
            if p.back {
                if let Some(mut b) = back
                    && b.0 != c
                {
                    b.0 = c;
                }
            } else if let Some(mut tc) = text
                && tc.0 != c
            {
                tc.0 = c;
            }
            if let Some(mut sh) = shade {
                sh.color = sh.color.with_alpha(p.shadow * a);
            }
        }
    }
}

#[derive(Default)]
struct MedalState {
    last: Option<u32>,
    queue: MedalQueue,
    /// The box drawn: its medal number and the window height it was built for.
    built: Option<(u64, u32)>,
}

/// BO2's `medal_received` notifies, shown as Black Ops' medal boxes.
#[allow(clippy::too_many_arguments)]
fn medal_notify(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    menus: Option<Res<frame::LuiMenus>>,
    (shell_fonts, game_font): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>),
    slot: NonSend<crate::lui_hud::LuiHudSlot>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<MedalRoot>>,
    time: Res<Time<Real>>,
    mut st: Local<MedalState>,
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
            *st = MedalState::default();
            return;
        }
    };
    let Some(seen) = crate::hud_match::read_seen(p, l, menus.as_deref()) else {
        clear(&mut commands);
        *st = MedalState::default();
        return;
    };
    let now = time.elapsed_secs_f64();
    let bo1 = bo1_strings();
    let lui = p
        .snapshot()
        .map(|s| {
            s.meta
                .script_dvars(l.0)
                .string("bo2mp_lui")
                .unwrap_or_default()
                .to_owned()
        })
        .unwrap_or_default();
    for row in new_medals(&lui, &mut st.last) {
        let Some(found) = slot
            .table_row(SCORE_INFO, row)
            .and_then(|(h, r)| medal_row(&h, &r, &seen.gametype))
        else {
            diag::warn!(
                Ui,
                "bo2mp hud: medal row {row}: not a medal in {SCORE_INFO} (not shown)"
            );
            continue;
        };
        let bo1_text = |k: &str| bo1.as_ref().and_then(|m| m.get(k).cloned());
        let bo2_text = |k: &str| slot.localize(k);
        let (model, source) = medal_model(&found, &bo1_text, &bo2_text);
        diag::info!(
            Ui,
            "bo2mp hud: medal row {row} {} ({}): \"{}\" / \"{}\" {} [{source:?} text; {} XP {} score in {}{}]",
            found.event,
            found.name_key,
            model.title,
            model.desc,
            model.points,
            found.xp,
            found.score,
            seen.gametype,
            if bo1.is_none() {
                "; Black Ops text not read yet"
            } else {
                ""
            }
        );
        if !st.queue.push(model) {
            diag::warn!(
                Ui,
                "bo2mp hud: medal {} dropped: {} already waiting",
                found.event,
                st.queue.waiting()
            );
        }
    }
    // (Time runs while the HUD is away: nothing waits to stick on screen
    // after a killcam or a menu.)
    let Some((model, start, life)) = st.queue.tick(now).map(|(m, s, l)| (m.clone(), s, l)) else {
        clear(&mut commands);
        st.built = None;
        return;
    };
    // (Each medal shown is the queue's next start.)
    let id = st.queue.started;
    if !seen.hud || seen.ended {
        clear(&mut commands);
        st.built = None;
        return;
    }
    let height = window.single().map_or(1080.0, Window::height);
    let key = (id, height as u32);
    if st.built == Some(key) && !roots.is_empty() {
        return;
    }
    let Some(faces) = Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else {
        return;
    };
    clear(&mut commands);
    spawn_medal(
        &mut commands,
        &faces,
        &model,
        height,
        MedalRoot { start, life },
    );
    st.built = Some(key);
}

pub(crate) fn register_medal_systems(app: &mut App) {
    app.add_systems(
        Update,
        (medal_notify, medal_animate).chain().in_set(ClientSet::Ui),
    );
}

// --- Preview ----------------------------------------------------------------------------------

pub mod preview {
    //! Black Ops' medal box with real medal rows, saved as pictures
    //! (`cargo run -p ui --example medal_preview -- <out dir>`). Black
    //! Ops' text is read from `IW4L_GAMES` when it is set.

    use std::path::PathBuf;

    use bevy::prelude::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    use super::*;

    /// BO2's `mp/scoreInfo.csv` header and medal rows, as the table has them
    /// (the columns up to `conf XP`).
    const HEADER: &str = "Script Reference ,Score String,Medal Reference,Medal Backing Material,Killstreaks Allowed Score,Save Medal Stats,Demo Bookmark Priority,DoubleXP,GunXP,Medal Category,Medal Sortkey,tdm score,tdm XP,dm score,dm XP,conf score,conf XP";
    const ROWS: &[&str] = &[
        "kill_enemy_one_bullet,,MEDAL_KILL_ENEMY_ONE_BULLET,hud_medals_oneshot_onekill,,,5,TRUE,,combat,16,0,100,0,100,0,100",
        "revenge_kill,,MEDAL_REVENGE_KILL,hud_medals_revenge,,TRUE,,TRUE,,combat,16,0,100,0,100,0,100",
        "multikill_2,,MEDAL_MULTIKILL_2,hud_medals_doublekill,,TRUE,4,TRUE,,combat,15,0,100,0,100,0,100",
        "first_kill,,MEDAL_FIRST_KILL,hud_medals_firstblood,,,4,TRUE,,combat,16,0,500,0,500,0,500",
        "retrieve_own_tags,SCORE_KILL_DENIED,MEDAL_RETRIEVE_OWN_TAGS,hud_medals_retrieval,,,4,TRUE,,gamemode,20,0,0,0,0,25,250",
        "killstreak_5,SCORE_KILLSTREAK_5,MEDAL_KILLSTREAK_5,hud_medals_bloodthirsty,,TRUE,4,TRUE,,combat,7,0,250,0,250,0,250",
    ];

    #[derive(Resource)]
    struct Plan {
        out: PathBuf,
        step: usize,
        frames: u32,
        /// The shown box's age to hold (s).
        age: f32,
        waited: u32,
    }

    #[derive(Component)]
    struct Shot;

    /// (name, rows shown (first drawn, rest waiting), game type, age, with the kill banner)
    const SHOTS: &[(&str, usize, &str, f32, bool)] = &[
        ("01_one_shot_one_kill", 0, "tdm", 0.6, false),
        ("02_payback_sliding_in", 1, "tdm", 0.07, false),
        ("03_double_kill_with_banner", 2, "tdm", 0.8, true),
        ("04_first_blood_fading_out", 3, "tdm", 1.62, false),
        ("05_kill_denied_conf_points", 4, "conf", 0.6, false),
        ("06_bo2_text_bloodthirsty", 5, "tdm", 0.6, false),
    ];

    fn model(i: usize, gametype: &str, bo1: Option<&HashMap<String, String>>) -> MedalModel {
        let header: Vec<String> = HEADER.split(',').map(str::to_owned).collect();
        let row: Vec<String> = ROWS[i].split(',').map(str::to_owned).collect();
        let r = medal_row(&header, &row, gametype).expect("a medal row");
        // (BO2's own English text is not read here: stand-in BO2 strings.)
        let bo2 = |k: &str| match k {
            "MEDAL_KILLSTREAK_5" => Some("Bloodthirsty".to_owned()),
            "MEDAL_KILLSTREAK_5_DESC" => Some("5 kills without dying.".to_owned()),
            "MEDAL_RETRIEVE_OWN_TAGS" => Some("Retrieval".to_owned()),
            "MEDAL_RETRIEVE_OWN_TAGS_DESC" => Some("Recovered a friendly tag.".to_owned()),
            _ => None,
        };
        let bo1t = |k: &str| bo1.and_then(|m| m.get(k).cloned());
        let (m, source) = medal_model(&r, &bo1t, &bo2);
        println!("medal preview: {} -> {:?} ({source:?})", r.event, m);
        m
    }

    #[allow(clippy::too_many_arguments)]
    fn step(
        mut commands: Commands,
        mut plan: ResMut<Plan>,
        fonts: Option<Res<ShellFonts>>,
        shots: Query<Entity, With<Shot>>,
        time: Res<Time<Real>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let Some(fonts) = fonts else { return };
        // (Black Ops' text first, when its zones are there: up to ~20 s.)
        let bo1 = bo1_strings();
        if bo1.is_none() && plan.waited < 1200 {
            plan.waited += 1;
            return;
        }
        let faces = Faces {
            regular: fonts.regular.clone(),
            bold: fonts.bold.clone(),
        };
        plan.frames += 1;
        if plan.frames < 30 {
            return;
        }
        plan.frames = 0;
        let i = plan.step / 2;
        if i >= SHOTS.len() {
            exit.write(AppExit::Success);
            return;
        }
        let (name, row, gametype, age, banner) = SHOTS[i];
        if plan.step % 2 == 1 {
            let path = plan.out.join(format!("{name}.png"));
            println!("medal preview: {}", path.display());
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
            plan.step += 1;
            return;
        }
        for e in &shots {
            commands.entity(e).despawn();
        }
        plan.age = age;
        let now = time.elapsed_secs_f64();
        let m = model(row, gametype, bo1.as_deref());
        let e = spawn_medal(
            &mut commands,
            &faces,
            &m,
            1080.0,
            MedalRoot {
                start: now - f64::from(age),
                life: MEDAL_LIFE,
            },
        );
        commands.entity(e).insert(Shot);
        if banner {
            let model = crate::hud_match::BannerModel {
                name: "Jimbo".into(),
                title: "Gun Runner".into(),
                level: 62,
                card: None,
                emblem: None,
                rank: None,
                start: now,
            };
            let e = crate::hud_match::spawn_banner(&mut commands, &faces, &model, 1080.0);
            commands.entity(e).insert(Shot);
        }
        // A plain stand-in scene, and a crosshair's place.
        let e = commands
            .spawn((
                BackgroundColor(Color::srgb(0.36, 0.37, 0.34)),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                GlobalZIndex(-1),
            ))
            .with_children(|s| {
                s.spawn((
                    BackgroundColor(Color::srgb(0.95, 0.95, 0.95)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(957.0),
                        top: Val::Px(537.0),
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        ..default()
                    },
                ));
            })
            .id();
        commands.entity(e).insert(Shot);
        plan.step += 1;
    }

    /// (The preview's frames are slow: each box is held at its age.)
    fn hold(time: Res<Time<Real>>, plan: Res<Plan>, mut q: Query<&mut MedalRoot>) {
        let now = time.elapsed_secs_f64();
        for mut r in &mut q {
            r.start = now - f64::from(plan.age);
        }
    }

    pub fn install(app: &mut App, out: PathBuf) {
        app.add_systems(Startup, crate::shell::theme::load_fonts);
        app.insert_resource(Plan {
            out,
            step: 0,
            frames: 0,
            age: 0.0,
            waited: 0,
        });
        app.add_systems(Update, (step, hold, super::medal_animate).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> Vec<String> {
        "Script Reference ,Score String,Medal Reference,Medal Backing Material,Killstreaks Allowed Score,Save Medal Stats,Demo Bookmark Priority,DoubleXP,GunXP,Medal Category,Medal Sortkey,tdm score,tdm XP,dm score,dm XP,conf score,conf XP"
            .split(',')
            .map(str::to_owned)
            .collect()
    }

    fn row(s: &str) -> Vec<String> {
        s.split(',').map(str::to_owned).collect()
    }

    fn medal(title: &str) -> MedalModel {
        MedalModel {
            title: title.into(),
            desc: String::new(),
            points: String::new(),
        }
    }

    #[test]
    fn rows_read_xp_by_game_type() {
        let h = header();
        let one = medal_row(&h, &row("kill_enemy_one_bullet,,MEDAL_KILL_ENEMY_ONE_BULLET,hud_medals_oneshot_onekill,,,5,TRUE,,combat,16,0,100,0,100,0,100"), "tdm").unwrap();
        assert_eq!(one.event, "kill_enemy_one_bullet");
        assert_eq!((one.xp, one.score, one.points()), (100, 0, Some(100)));
        let tags = row(
            "retrieve_own_tags,SCORE_KILL_DENIED,MEDAL_RETRIEVE_OWN_TAGS,hud_medals_retrieval,,,4,TRUE,,gamemode,20,0,0,0,0,25,250",
        );
        assert_eq!(medal_row(&h, &tags, "conf").unwrap().points(), Some(250));
        assert_eq!(medal_row(&h, &tags, "tdm").unwrap().points(), None);
        // A game type without columns of its own reads TDM's.
        assert_eq!(medal_row(&h, &tags, "zzz").unwrap().points(), None);
        // A row that is no medal (no Medal Reference).
        assert!(
            medal_row(
                &h,
                &row("kill,SCORE_KILL,,,,,,TRUE,TRUE,,,100,100,100,100,50,100"),
                "tdm"
            )
            .is_none()
        );
    }

    #[test]
    fn text_bo1_then_bo2_then_name() {
        let h = header();
        let r = medal_row(&h, &row("kill_enemy_one_bullet,,MEDAL_KILL_ENEMY_ONE_BULLET,x,,,5,TRUE,,combat,16,0,100,0,100,0,100"), "tdm").unwrap();
        let bo1 = |k: &str| {
            Some(
                match k {
                    "MEDAL_ONE_SHOT_KILL" => "One Shot, One Kill",
                    "MEDAL_ONE_SHOT_KILL_DESC" => "Killed an enemy with one shot.",
                    "MEDAL_XP_POPUP" => "^3+&&1^7",
                    _ => return None,
                }
                .to_owned(),
            )
        };
        let bo2 = |k: &str| Some(format!("bo2 {k}"));
        let none = |_: &str| None;
        let (m, s) = medal_model(&r, &bo1, &bo2);
        assert_eq!(s, TextSource::Bo1);
        assert_eq!(
            m,
            MedalModel {
                title: "One Shot, One Kill".into(),
                desc: "Killed an enemy with one shot.".into(),
                points: "+100".into()
            }
        );
        // No Black Ops text: BO2's, and the plain "+N".
        let (m, s) = medal_model(&r, &none, &bo2);
        assert_eq!(s, TextSource::Bo2);
        assert_eq!(
            (m.title.as_str(), m.desc.as_str(), m.points.as_str()),
            (
                "bo2 MEDAL_KILL_ENEMY_ONE_BULLET",
                "bo2 MEDAL_KILL_ENEMY_ONE_BULLET_DESC",
                "+100"
            )
        );
        // Neither game's: the event's name, no description.
        let (m, s) = medal_model(&r, &none, &none);
        assert_eq!(s, TextSource::Name);
        assert_eq!(
            (m.title.as_str(), m.desc.as_str()),
            ("Kill Enemy One Bullet", "")
        );
        // A medal Black Ops did not have keeps BO2's even with BO1 text there.
        let ks = medal_row(&h, &row("killstreak_5,SCORE_KILLSTREAK_5,MEDAL_KILLSTREAK_5,x,,TRUE,4,TRUE,,combat,7,0,250,0,250,0,250"), "tdm").unwrap();
        let (m, s) = medal_model(&ks, &bo1, &bo2);
        assert_eq!((s, m.points.as_str()), (TextSource::Bo2, "+250"));
        // Points come from the row, never a fixed number.
        assert_ne!(m.points, "+50");
    }

    #[test]
    fn colour_codes_drop() {
        assert_eq!(plain("^3+50^7"), "+50");
        assert_eq!(plain("Boom, ^1headshot^7."), "Boom, headshot.");
        assert_eq!(plain("a^b"), "a^b");
    }

    #[test]
    fn notifies_once_each_and_reset() {
        let mut last = None;
        // The first look: what is there is old.
        assert!(
            new_medals(
                "3|score_event|SCORE_KILL|100|0;4|medal_received|123",
                &mut last
            )
            .is_empty()
        );
        let feed = "3|score_event|SCORE_KILL|100|0;4|medal_received|123;5|score_event|SCORE_KILL|100|0;6|medal_received|156;7|medal_received|115";
        assert_eq!(new_medals(feed, &mut last), vec![156, 115]);
        // The same feed again: nothing new (never twice).
        assert!(new_medals(feed, &mut last).is_empty());
        // A new match counts from 1 again.
        assert_eq!(
            new_medals(
                "1|faction_popup|MP_FBI_NAME;2|medal_received|107",
                &mut last
            ),
            vec![107]
        );
        assert!(
            new_medals(
                "1|faction_popup|MP_FBI_NAME;2|medal_received|107",
                &mut last
            )
            .is_empty()
        );
        // Junk is passed over.
        assert!(new_medals("x|medal_received|1;9|medal_received|", &mut last).is_empty());
    }

    #[test]
    fn queue_shows_one_after_another_and_never_sticks() {
        let mut q = MedalQueue::default();
        assert!(q.tick(0.0).is_none());
        q.push(medal("a"));
        q.push(medal("b"));
        let (m, start, life) = q
            .tick(1.0)
            .map(|(m, s, l)| (m.title.clone(), s, l))
            .unwrap();
        assert_eq!((m.as_str(), start, life), ("a", 1.0, MEDAL_LIFE));
        assert_eq!(
            q.tick(1.0 + f64::from(MEDAL_LIFE) - 0.01).unwrap().0.title,
            "a"
        );
        // "b" right after "a", from then.
        let t = 1.0 + f64::from(MEDAL_LIFE);
        let (m, start, _) = q.tick(t).map(|(m, s, l)| (m.title.clone(), s, l)).unwrap();
        assert_eq!((m.as_str(), start), ("b", t));
        // Gone after its life: nothing left on screen.
        assert!(q.tick(t + f64::from(MEDAL_LIFE)).is_none());
        assert!(q.tick(t + 100.0).is_none());
        assert_eq!(q.started, 2);
    }

    #[test]
    fn queue_hurries_a_backlog_and_caps_it() {
        let mut q = MedalQueue::default();
        for i in 0..MAX_WAITING {
            assert!(q.push(medal(&i.to_string())));
        }
        assert!(!q.push(medal("over")));
        let (_, _, life) = q.tick(0.0).unwrap();
        assert_eq!(life, QUICK_LIFE);
        // Each in turn, then none.
        let mut now = 0.0;
        while let Some((_, s, l)) = q.tick(now) {
            now = s + f64::from(l);
        }
        assert_eq!(q.started, MAX_WAITING as u64);
        // The last ones, with fewer waiting, get the whole life.
        let mut q = MedalQueue::default();
        q.push(medal("x"));
        assert_eq!(q.tick(0.0).unwrap().2, MEDAL_LIFE);
    }

    #[test]
    fn motion_fades_in_holds_and_out() {
        let (a0, y0, s0) = medal_motion(0.0, MEDAL_LIFE);
        assert!(a0 < 0.01 && y0 < -10.0 && s0 > 1.05);
        let (a, y, s) = medal_motion(0.8, MEDAL_LIFE);
        assert!((a - 1.0).abs() < 1e-6 && y.abs() < 1e-6 && (s - 1.0).abs() < 1e-6);
        let (a, y, _) = medal_motion(MEDAL_LIFE - 0.1, MEDAL_LIFE);
        assert!(a < 0.5 && y < 0.0);
        assert!(medal_motion(MEDAL_LIFE, MEDAL_LIFE).0 < 1e-6);
    }
}
