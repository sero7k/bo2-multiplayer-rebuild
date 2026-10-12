//! bo2mp shell: what the front end's scripts know, handed to the shell, and
//! what the shell asks of them.
//!
//! The shell draws every front-end screen itself. BO2's scripts still run
//! underneath (his stats file, Create-a-Class, the options, the match's
//! start and its countdown); `lui_hud` calls `lui_side` once a frame with
//! their host, which publishes the catalog (maps, modes, his card) and
//! carries out the shell's requests.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use hks_t6::host::Host;
use hks_t6::mp::Mp;
use hks_t6::value::Value;

/// A map the lobby can pick.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MapEntry {
    /// The name the scripts and the match setup know it by (`mp_la`,
    /// `mp_rust`).
    pub ui_name: String,
    pub title: String,
    /// The section it is listed under ("BLACK OPS II", "CLASSIC MAPS").
    pub group: String,
    /// Where it comes from ("Modern Warfare 2 (DLC)").
    pub origin: String,
    /// Picture names to try, best first.
    pub pictures: Vec<String>,
}

/// A game mode the lobby can pick.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ModeEntry {
    pub key: String,
    pub title: String,
    pub desc: String,
}

/// His card: name, level, prestige and how far into the level he is.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PlayerCard {
    pub name: String,
    pub level: i32,
    pub prestige: i32,
    pub xp_frac: f32,
    /// The rank icon's picture name (another game's as `t5:<material>`).
    pub rank_picture: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Catalog {
    pub maps: Vec<MapEntry>,
    pub modes: Vec<ModeEntry>,
    pub player: PlayerCard,
}

impl Catalog {
    pub fn map(&self, ui_name: &str) -> Option<&MapEntry> {
        self.maps.iter().find(|m| m.ui_name.eq_ignore_ascii_case(ui_name))
    }

    pub fn mode(&self, key: &str) -> Option<&ModeEntry> {
        self.modes.iter().find(|m| m.key.eq_ignore_ascii_case(key))
    }

    /// The groups in the order their first map is listed.
    pub fn groups(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for m in &self.maps {
            if !out.contains(&m.group) {
                out.push(m.group.clone());
            }
        }
        out
    }
}

/// Which of BO2's own screens the shell opens over itself for now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LuiScreen {
    CreateAClass,
    Barracks,
    Options,
}

impl LuiScreen {
    /// `LUI.createMenu` names to try, in order.
    fn candidates(self) -> &'static [&'static str] {
        match self {
            Self::CreateAClass => &["CAC", "CACMain", "CACChooseClass", "ChooseClass", "ClassSelect", "EditClass"],
            Self::Barracks => &["Barracks", "BarracksMenu", "PlayerCard", "CombatRecord"],
            Self::Options => &["OptionsMenu", "Options", "OptionsMain", "SettingsMenu", "OptionsSettings"],
        }
    }
}

/// What the match he starts plays.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StartSpec {
    pub map: String,
    pub gametype: String,
    pub bot_friends: u32,
    pub bot_enemies: u32,
    pub bot_difficulty: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Request {
    Start(StartSpec),
    CancelStart,
    Open(LuiScreen),
    Zombies,
}

#[derive(Default)]
struct Bridge {
    catalog: Option<Arc<Catalog>>,
    requests: Vec<Request>,
    /// A BO2 screen he is in: the menus open before it (it is closed when
    /// they are back), whether they have changed since.
    lui: Option<(Vec<String>, bool)>,
    /// The match starts in this many ms (the private countdown).
    starting_in: Option<f64>,
    /// A line for him (a BO2 screen that could not open).
    note: Option<String>,
    built_at: f64,
}

static BRIDGE: Mutex<Option<Bridge>> = Mutex::new(None);
static LUI_OPEN: AtomicBool = AtomicBool::new(false);
static SHOWN: AtomicBool = AtomicBool::new(false);
static LOADING_SHOWN: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut Bridge) -> R) -> R {
    let mut guard = BRIDGE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(guard.get_or_insert_with(Bridge::default))
}

/// The shell draws the front end (`BO2_SHELL=off`: BO2's own menus).
pub(crate) fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| !std::env::var("BO2_SHELL").is_ok_and(|v| v.eq_ignore_ascii_case("off") || v == "0"))
}

/// The shell's screens are up: BO2's menus are hidden and get no input.
pub(crate) fn covers_lui() -> bool {
    enabled() && SHOWN.load(Ordering::Relaxed) && !LUI_OPEN.load(Ordering::Relaxed)
}

/// The shell's loading screen is up (BO2's is not opened).
pub(crate) fn covers_loading() -> bool {
    enabled() && LOADING_SHOWN.load(Ordering::Relaxed)
}

pub(crate) fn set_shown(on: bool) {
    SHOWN.store(on, Ordering::Relaxed);
}

pub(crate) fn set_loading_shown(on: bool) {
    LOADING_SHOWN.store(on, Ordering::Relaxed);
}

/// One of BO2's screens is open over the shell.
pub(crate) fn lui_open() -> bool {
    LUI_OPEN.load(Ordering::Relaxed)
}

pub(crate) fn catalog() -> Option<Arc<Catalog>> {
    with(|b| b.catalog.clone())
}

/// The preview's stand-in catalog.
pub(crate) fn set_preview_catalog(c: Catalog) {
    with(|b| b.catalog = Some(Arc::new(c)));
}

pub(crate) fn request(r: Request) {
    with(|b| b.requests.push(r));
}

pub(crate) fn starting_in() -> Option<f64> {
    with(|b| b.starting_in)
}

pub(crate) fn take_note() -> Option<String> {
    with(|b| b.note.take())
}

fn text(v: &hks_t6::host::EngineValues, key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    let k = key.to_ascii_uppercase();
    v.localize.get(&k).cloned().unwrap_or_default()
}

fn cell(row: &[String], i: usize) -> &str {
    row.get(i).map_or("", |c| c.trim())
}

/// A picture name from a mapstable cell: a material-looking word.
fn looks_like_picture(c: &str) -> bool {
    !c.is_empty() && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') && c.chars().any(|ch| ch.is_ascii_alphabetic()) && c.contains('_')
}

fn build_catalog(host: &Host, mp: Option<&Mp>) -> Catalog {
    let v = host.values.borrow();
    // Maps: BO2's mapstable (its own rows, then the other games' the map
    // list added), in the order of its sort column.
    let foreign = crate::foreign_maps::categories();
    let mut maps: Vec<(i32, MapEntry)> = Vec::new();
    for row in hks_t6::mp::table_rows(&v, "mp/mapstable.csv") {
        let name = cell(row, 0);
        if name.is_empty() || !name.starts_with("mp_") || maps.iter().any(|(_, m)| m.ui_name == name) {
            continue;
        }
        let key = cell(row, 3);
        let title = [format!("{key}_CAPS"), key.to_owned()].iter().map(|k| text(&v, k)).find(|t| !t.is_empty());
        let Some(title) = title else { continue };
        let sort = cell(row, 5).parse::<i32>().unwrap_or(999);
        let (group, origin) = foreign
            .iter()
            .find_map(|c| c.maps.iter().find(|m| m.ui_name == name).map(|m| (c.title.clone(), m.origin.clone())))
            .unwrap_or_else(|| ("BLACK OPS II".to_owned(), "Black Ops II".to_owned()));
        // (A foreign map's row is Aftermath's, renamed: its icon column is
        // Aftermath's picture, so only BO2's own maps use it.)
        let own = group == "BLACK OPS II";
        let mut pictures = Vec::new();
        if own && looks_like_picture(cell(row, 4)) {
            pictures.push(cell(row, 4).to_owned());
        }
        pictures.extend([
            format!("menu_{name}_map_select_final"),
            format!("loadscreen_{name}"),
            format!("menu_{name}_map_select"),
            format!("menu_mp_map_select_{}_big", name.trim_start_matches("mp_")),
        ]);
        maps.push((
            if group == "BLACK OPS II" { sort } else { 10_000 + sort },
            MapEntry {
                ui_name: name.to_owned(),
                title: title.to_uppercase(),
                group,
                origin,
                pictures,
            },
        ));
    }
    maps.sort_by_key(|(s, _)| *s);
    // Modes: gametypestable's multiplayer rows with a name.
    let mut modes: Vec<ModeEntry> = Vec::new();
    let mut rows: Vec<&Vec<String>> = hks_t6::mp::table_rows(&v, "mp/gametypestable.csv").iter().filter(|r| cell(r, 0) == "0").collect();
    rows.sort_by_key(|r| cell(r, 5).parse::<i32>().unwrap_or(999));
    for row in rows {
        let key = cell(row, 1);
        let title = text(&v, cell(row, 2));
        if key.is_empty() || title.is_empty() || modes.iter().any(|m| m.key == key) {
            continue;
        }
        let desc = [cell(row, 3)].iter().map(|k| text(&v, k)).find(|t| !t.is_empty()).unwrap_or_default();
        modes.push(ModeEntry {
            key: key.to_owned(),
            title: title.to_uppercase(),
            desc,
        });
    }
    // His card.
    let name = v.dvars.get("name").filter(|n| !n.trim().is_empty()).cloned().unwrap_or_else(|| "Player".to_owned());
    let (rank, prestige, xp) = mp.map_or((0, 0, 0.0), |m| {
        let m = m.borrow();
        (
            m.profile.rank().max(0),
            m.profile.plevel().max(0),
            m.stat(bo2_profile::RANK_XP).as_num().unwrap_or(0.0),
        )
    });
    let xp_frac = hks_t6::mp::table_rows(&v, "mp/ranktable.csv")
        .iter()
        .chain(hks_t6::mp::table_rows(&v, "mp/rankTable.csv"))
        .find(|r| cell(r, 0).parse::<i32>().ok() == Some(rank))
        .and_then(|r| {
            let lo = cell(r, 2).parse::<f32>().ok()?;
            let hi = cell(r, 7).parse::<f32>().ok()?;
            (hi > lo).then(|| ((xp - lo) / (hi - lo)).clamp(0.0, 1.0))
        })
        .unwrap_or(0.0);
    let level = rank + 1;
    Catalog {
        maps: maps.into_iter().map(|(_, m)| m).collect(),
        modes,
        player: PlayerCard {
            name,
            level,
            prestige,
            xp_frac,
            rank_picture: bo2_profile::progression::rank_picture(level, prestige),
        },
    }
}

/// The names `LUI.createMenu` holds (logged once, to find a screen's name).
fn menu_names(host: &mut Host) -> Vec<String> {
    let made = host.field("LUI", "createMenu");
    let mut out = Vec::new();
    if let Some(t) = made.table() {
        let t = t.borrow();
        let mut k = Value::Nil;
        while let Some((key, _)) = t.next(&k) {
            if let Some(s) = key.as_str() {
                out.push(s.to_owned());
            }
            k = key;
        }
    }
    out.sort();
    out
}

fn has_menu(host: &mut Host, name: &str) -> bool {
    let made = host.field("LUI", "createMenu");
    !matches!(host.vm.index(&made, &Value::str(name)), Ok(Value::Nil) | Err(_))
}

/// Once a frame in the front end, with BO2's scripts: the catalog out, the
/// shell's requests in.
pub(crate) fn lui_side(host: &mut Host, mp: Option<&Mp>) {
    if !enabled() {
        return;
    }
    let now = host.now_ms();
    // The catalog, again every two seconds (his card changes after a match).
    let rebuild = with(|b| b.catalog.is_none() || now - b.built_at > 2000.0 || now < b.built_at);
    if rebuild {
        let c = build_catalog(host, mp);
        let first = with(|b| b.catalog.is_none());
        if first {
            diag::info!(
                Ui,
                "bo2mp shell: {} maps in {} groups, {} modes; {} level {} prestige {} ({})",
                c.maps.len(),
                c.groups().len(),
                c.modes.len(),
                c.player.name,
                c.player.level,
                c.player.prestige,
                c.player.rank_picture
            );
            if let Some(m) = c.maps.first() {
                diag::info!(Ui, "bo2mp shell: first map {} pictures tried {:?}", m.ui_name, m.pictures);
            }
        }
        with(|b| {
            if b.catalog.as_deref() != Some(&c) {
                b.catalog = Some(Arc::new(c));
            }
            b.built_at = now;
        });
    }
    // The private countdown.
    let starting = mp.and_then(|m| m.borrow().private_start_ms).map(|at| (at - now).max(0.0));
    let requested = mp.is_some_and(|m| m.borrow().start_requested);
    with(|b| b.starting_in = starting.or(requested.then_some(5000.0)));
    // A BO2 screen he is in: done when the menus are back as they were.
    let open = host.open_menus();
    let back = with(|b| match b.lui.as_mut() {
        Some((before, changed)) => {
            if open != *before {
                *changed = true;
                false
            } else {
                *changed
            }
        }
        None => false,
    });
    if back {
        with(|b| b.lui = None);
        LUI_OPEN.store(false, Ordering::Relaxed);
        diag::info!(Ui, "bo2mp shell: back from BO2's screen");
    }
    for r in with(|b| std::mem::take(&mut b.requests)) {
        match r {
            Request::Start(spec) => {
                {
                    let mut v = host.values.borrow_mut();
                    for (k, x) in [
                        ("ui_mapname", spec.map.clone()),
                        ("ui_gametype", spec.gametype.clone()),
                        ("bot_friends", spec.bot_friends.to_string()),
                        ("bot_enemies", spec.bot_enemies.to_string()),
                        ("bot_difficulty", spec.bot_difficulty.to_string()),
                    ] {
                        v.dvars.insert(k.to_owned(), x);
                    }
                }
                if let Some(m) = mp {
                    let mut m = m.borrow_mut();
                    m.game_modes.insert(1);
                    m.start_requested = true;
                }
                diag::info!(Ui, "bo2mp shell: START {spec:?}");
            }
            Request::CancelStart => {
                if let Some(m) = mp {
                    let mut m = m.borrow_mut();
                    m.start_requested = false;
                    m.private_start_ms = None;
                }
            }
            Request::Zombies => {
                if let Some(m) = mp {
                    m.borrow_mut().zombies_requested = true;
                }
            }
            Request::Open(screen) => {
                static LOGGED: AtomicBool = AtomicBool::new(false);
                if !LOGGED.swap(true, Ordering::Relaxed) {
                    let names = menu_names(host);
                    diag::info!(Ui, "bo2mp shell: BO2 screens available ({}): {}", names.len(), names.join(" "));
                }
                let found = screen.candidates().iter().copied().find(|n| has_menu(host, n));
                match found {
                    Some(name) => {
                        with(|b| b.lui = Some((open.clone(), false)));
                        LUI_OPEN.store(true, Ordering::Relaxed);
                        host.open_menu(name);
                        diag::info!(Ui, "bo2mp shell: opened BO2's `{name}` for {screen:?}");
                    }
                    None => {
                        diag::warn!(Ui, "bo2mp shell: none of {:?} is a BO2 screen", screen.candidates());
                        with(|b| b.note = Some("That screen is not available yet.".to_owned()));
                    }
                }
            }
        }
    }
}
