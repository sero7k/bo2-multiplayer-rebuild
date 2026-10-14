//! bo2mp shell: Zombies. The ZOMBIES row opens the shell's own screens in
//! red: a lobby (the map, its start and mode, the starting round, START), a
//! map picker and a start picker. START runs the Zombies rebuild
//! (DaffyDabz/bo2-zombies-rebuild, a sibling fork of this engine that plays
//! every Black Ops II Zombies map and, with the Zombies Declassified pack,
//! nine Black Ops maps) straight into the chosen game:
//!
//! ```text
//! <zombies>\play\iw4l.exe map t6:zm_transit   (cwd: <zombies>\play)
//!     IW4L_T6_START=town:zgrief               the start and game type
//!     IW4L_T6_START_ROUND=10                  the starting round (when > 1)
//! ```
//!
//! Its program is found by `BO2MP_ZOMBIES_EXE` (the build script writes it
//! to this game's `.env`), else at `..\zombies\play\iw4l.exe` beside this
//! game's play folder. Our window is minimised while it runs (it gets the
//! screen, the GPU and the sound to itself: our updates drop to four a
//! second and our music pauses), the shell keeps an IN ZOMBIES card up,
//! and when the Zombies game closes our window comes back on the Zombies
//! lobby.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{Act, Row, Screen, Setting, Shell};

/// A place to start on a map, and the mode played there.
#[derive(Debug, PartialEq)]
pub(crate) struct ZmStart {
    pub name: &'static str,
    pub mode_title: &'static str,
    /// The start location and game type (`ui_zm_mapstartlocation`,
    /// `g_gametype`); empty: the map's own (the Black Ops maps).
    pub loc: &'static str,
    pub mode: &'static str,
    /// The zone it loads when that is not the map's own (Turned at the
    /// Diner is `zm_transit_dr`, as BO2's start menu loads it).
    pub zone: Option<&'static str>,
    pub desc: &'static str,
}

/// A Zombies map.
#[derive(Debug, PartialEq)]
pub(crate) struct ZmMap {
    pub map: &'static str,
    pub title: &'static str,
    pub place: &'static str,
    /// A Black Ops map (Zombies Declassified's port): listed only when the
    /// Zombies game has the pack (`IW4L_T6_EXTRA` in its `.env`).
    pub bo1: bool,
    /// The Zombies rebuild's own screenshot of it (`docs/screenshots`).
    pub shot: &'static str,
    pub starts: &'static [ZmStart],
}

const fn at(name: &'static str, mode_title: &'static str, loc: &'static str, mode: &'static str, desc: &'static str) -> ZmStart {
    ZmStart { name, mode_title, loc, mode, zone: None, desc }
}

const STORY: &str = "STORY";
const SURVIVAL: &str = "SURVIVAL";
const GRIEF: &str = "GRIEF";
const TURNED: &str = "TURNED";

const GRIEF_DESC: &str = "Two teams, one map: outlast the other side. You can't shoot them, but you can make the zombies' job easier.";
const SURVIVAL_DESC: &str = "Endless rounds in one part of the map. No bus, no story: hold out as long as you can.";

/// One BO1 map: its own story mode, at its own start.
const fn bo1(map: &'static str, title: &'static str, place: &'static str, shot: &'static str, starts: &'static [ZmStart]) -> ZmMap {
    ZmMap { map, title, place, bo1: true, shot, starts }
}

const BO1_STORY: &[ZmStart] = &[at("STORY", STORY, "", "zclassic", "The whole map, by Black Ops' rules: doors, the box, perks and Pack-a-Punch.")];

/// Every map the Zombies rebuild plays, in Black Ops II's globe order,
/// then the Black Ops maps (its README's table).
pub(crate) const MAPS: &[ZmMap] = &[
    ZmMap {
        map: "zm_transit",
        title: "TRANZIT",
        place: "Hagerstown: a bus route through the fog",
        bo1: false,
        shot: "zm_transit",
        starts: &[
            at("TRANZIT", STORY, "transit", "zclassic", "The whole map: ride the bus from the depot to the diner, the farm, the power station and town; build, turn the power on."),
            at("BUS DEPOT", SURVIVAL, "transit", "zstandard", SURVIVAL_DESC),
            at("TOWN", SURVIVAL, "town", "zstandard", SURVIVAL_DESC),
            at("TOWN", GRIEF, "town", "zgrief", GRIEF_DESC),
            at("FARM", SURVIVAL, "farm", "zstandard", SURVIVAL_DESC),
            at("FARM", GRIEF, "farm", "zgrief", GRIEF_DESC),
            ZmStart {
                name: "DINER",
                mode_title: TURNED,
                loc: "diner",
                mode: "zcleansed",
                zone: Some("zm_transit_dr"),
                desc: "You play the zombie: kill the one human left to take his place, and stay human as long as you can.",
            },
        ],
    },
    ZmMap {
        map: "zm_nuked",
        title: "NUKETOWN",
        place: "The test town, with a warhead on the way",
        bo1: false,
        shot: "zm_nuked",
        starts: &[at("NUKETOWN", SURVIVAL, "nuked", "zstandard", "The test town, perks falling from the sky. Hold out round after round before the bombs fall.")],
    },
    ZmMap {
        map: "zm_highrise",
        title: "DIE RISE",
        place: "Shanghai: the broken towers",
        bo1: false,
        shot: "zm_highrise",
        starts: &[at("DIE RISE", STORY, "rooftop", "zclassic", "Wrecked skyscrapers, elevators and the drop between them: the Jet Gun, the Sliquifier and the Trample Steam.")],
    },
    ZmMap {
        map: "zm_prison",
        title: "MOB OF THE DEAD",
        place: "Alcatraz, 1933",
        bo1: false,
        shot: "zm_prison",
        starts: &[
            at("MOB OF THE DEAD", STORY, "prison", "zclassic", "Four mobsters on the Rock: Afterlife, Cerberus and the plane to build."),
            at("CELL BLOCK", GRIEF, "cellblock", "zgrief", GRIEF_DESC),
        ],
    },
    ZmMap {
        map: "zm_buried",
        title: "BURIED",
        place: "An old west town, deep underground",
        bo1: false,
        shot: "zm_buried",
        starts: &[
            at("BURIED", STORY, "processing", "zclassic", "The buried town: the Giant, the Ghosts of the mansion and the Paralyzer."),
            at("BOROUGH", GRIEF, "street", "zgrief", GRIEF_DESC),
            at("BOROUGH", TURNED, "street", "zcleansed", "You play the zombie: kill the one human left to take his place, and stay human as long as you can."),
        ],
    },
    ZmMap {
        map: "zm_tomb",
        title: "ORIGINS",
        place: "Northern France, 1918",
        bo1: false,
        shot: "zm_tomb",
        starts: &[at("ORIGINS", STORY, "tomb", "zclassic", "The trenches and the dig site: giant robots, the generators and the four elemental staffs.")],
    },
    bo1("zm_prototype", "NACHT DER UNTOTEN", "A bunker in the ruins", "zm_bo1_nacht", BO1_STORY),
    bo1("zm_asylum", "VERRÜCKT", "Wittenau Sanatorium", "zm_bo1_verruckt", BO1_STORY),
    bo1("zm_sumpf", "SHI NO NUMA", "A swamp in Japan", "zm_bo1_numa", BO1_STORY),
    bo1("zm_factory", "DER RIESE", "The factory at Breslau", "zm_bo1_riese", BO1_STORY),
    bo1("zm_theater", "KINO DER TOTEN", "A theatre in Berlin", "zm_bo1_kino", BO1_STORY),
    bo1("zm_pentagon", "\"FIVE\"", "The Pentagon", "zm_bo1_five", BO1_STORY),
    bo1("zm_cosmodrome", "ASCENSION", "A Soviet cosmodrome", "zm_bo1_ascension", BO1_STORY),
    bo1("zm_temple", "SHANGRI-LA", "A temple in the Himalayas", "zm_bo1_shangri", BO1_STORY),
    bo1("zm_moon", "MOON", "Griffin Station", "zm_bo1_moon", BO1_STORY),
];

/// The starting rounds offered (Black Ops II's Custom Games steps).
pub(crate) const ROUNDS: [u32; 5] = [1, 5, 10, 15, 20];

pub(crate) fn map(key: &str) -> &'static ZmMap {
    MAPS.iter().find(|m| m.map == key).unwrap_or(&MAPS[0])
}

/// The map picker's pages: (title, maps by index into `MAPS`).
pub(crate) fn pages(bo1: bool) -> Vec<(&'static str, Vec<usize>)> {
    let pick = |want: bool| MAPS.iter().enumerate().filter(|(_, m)| m.bo1 == want).map(|(i, _)| i).collect::<Vec<_>>();
    let mut out = vec![("BLACK OPS II", pick(false))];
    if bo1 {
        out.push(("BLACK OPS", pick(true)));
    }
    out
}

/// A start's name (a Black Ops map's one start is the map's own).
pub(crate) fn start_name(m: &ZmMap, s: &ZmStart) -> &'static str {
    if m.bo1 { m.title } else { s.name }
}

/// A start's line in a list: `TOWN · GRIEF` (a story mode is the map's
/// own name).
pub(crate) fn start_label(m: &ZmMap, s: &ZmStart) -> String {
    if s.mode_title == STORY { start_name(m, s).to_owned() } else { format!("{}  ·  {}", s.name, s.mode_title) }
}

/// The game a start plays: `TOWN · GRIEF on TRANZIT`, `ORIGINS (STORY)`.
pub(crate) fn game_line(m: &ZmMap, s: &ZmStart) -> String {
    if s.mode_title == STORY { format!("{} (STORY)", m.title) } else { format!("{}  ·  {} on {}", s.name, s.mode_title, m.title) }
}

/// The chosen start (kept to the map's).
pub(crate) fn chosen(shell: &Shell) -> (&'static ZmMap, &'static ZmStart) {
    let m = map(&shell.zm_map);
    (m, &m.starts[shell.zm_start.min(m.starts.len() - 1)])
}

pub(crate) fn rows(s: &Shell) -> Vec<Row> {
    let st = state();
    let (m, z) = chosen(s);
    match s.screen {
        Screen::Zombies => {
            let mut r = vec![
                Row::new("START GAME", &format!("{}. You come back here when you quit Zombies.", game_line(m, z)), Act::ZmPlay),
                Row::new(
                    "CHANGE MAP",
                    if st.bo1() { "Every Black Ops II Zombies map, and the nine Black Ops maps." } else { "Every Black Ops II Zombies map." },
                    Act::ZmPickMap,
                ),
                Row::new("CHANGE START", "Where you start, and the mode: story, Survival, Grief or Turned.", Act::ZmPickStart),
                Row::new("STARTING ROUND", "The round the game begins on.", Act::Cycle(Setting::ZmRound)),
            ];
            r[3].value = Some(ROUNDS[s.zm_round.min(ROUNDS.len() - 1)].to_string());
            r
        }
        Screen::ZmMapPick => {
            let p = pages(st.bo1());
            p.get(s.page.min(p.len() - 1))
                .map(|(_, maps)| maps.iter().enumerate().map(|(k, &i)| Row::new(MAPS[i].title, MAPS[i].place, Act::Choose(k))).collect())
                .unwrap_or_default()
        }
        Screen::ZmStartPick => m.starts.iter().enumerate().map(|(i, z)| Row::new(&start_label(m, z), z.desc, Act::Choose(i))).collect(),
        _ => Vec::new(),
    }
}

/// CHANGE MAP: the picker on the page and tile of the map he has.
pub(crate) fn open_map_pick(s: &mut Shell) {
    let p = pages(state().bo1());
    let at = MAPS.iter().position(|m| m.map == s.zm_map).unwrap_or(0);
    let (page, k) = p.iter().enumerate().find_map(|(pi, (_, maps))| maps.iter().position(|&m| m == at).map(|k| (pi, k))).unwrap_or((0, 0));
    s.go(Screen::ZmMapPick);
    s.page = page;
    s.focus = k;
}

/// A map tile picked: that map, at its first start if it is a new one.
pub(crate) fn choose_map(s: &mut Shell, k: usize) {
    let p = pages(state().bo1());
    if let Some(&i) = p.get(s.page).and_then(|(_, maps)| maps.get(k))
        && MAPS[i].map != s.zm_map
    {
        s.zm_map = MAPS[i].map.to_owned();
        s.zm_start = 0;
    }
}

// ---------------------------------------------------------------------------
// The command line.

/// What START runs: the arguments and the environment it adds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ZmLaunch {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

/// The Zombies game's command line for a map's start and a starting round:
/// `map t6:<zone>` (its documented way past the menus), the start and mode
/// as `IW4L_T6_START=<location>:<game type>` (what its front end's Start
/// Match sets; read when the map loads), and the round as
/// `IW4L_T6_START_ROUND` (its `getgametypesetting("startround")`).
pub(crate) fn launch_spec(m: &ZmMap, start: usize, round: u32) -> ZmLaunch {
    let s = &m.starts[start.min(m.starts.len() - 1)];
    let zone = s.zone.unwrap_or(m.map);
    let mut env = Vec::new();
    if !s.loc.is_empty() {
        env.push(("IW4L_T6_START".to_owned(), format!("{}:{}", s.loc, s.mode)));
    }
    if round > 1 {
        env.push(("IW4L_T6_START_ROUND".to_owned(), round.to_string()));
    }
    ZmLaunch { args: vec!["map".to_owned(), format!("t6:{zone}")], env }
}

// ---------------------------------------------------------------------------
// The Zombies game on this PC.

/// Where the Zombies game is.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Install {
    pub exe: PathBuf,
    /// Its play folder (its `.env`, settings and logs; it starts there).
    pub dir: PathBuf,
    /// Its `.env` names a Zombies Declassified folder (`IW4L_T6_EXTRA`).
    pub extra: Vec<PathBuf>,
    /// Its source's screenshots (`<zombies>\source\docs\screenshots`).
    pub shots: Option<PathBuf>,
}

/// What the Zombies screens show about it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ZmState {
    pub install: Option<Result<Install, String>>,
    /// The game running: its line (`TRANZIT · TOWN (GRIEF)`).
    pub running: Option<String>,
}

impl ZmState {
    pub fn bo1(&self) -> bool {
        matches!(&self.install, Some(Ok(i)) if !i.extra.is_empty())
    }

    pub fn problem(&self) -> Option<&str> {
        match &self.install {
            Some(Err(e)) => Some(e.as_str()),
            _ => None,
        }
    }
}

static STATE: Mutex<Option<ZmState>> = Mutex::new(None);
static CHILD: Mutex<Option<(std::process::Child, std::time::Instant, PathBuf)>> = Mutex::new(None);
/// The preview: a stand-in install, nothing is started.
static PREVIEW: AtomicBool = AtomicBool::new(false);
/// bo2 merge: the `map` command START left for `zombies_exec`.
static PENDING_MAP: Mutex<Option<String>> = Mutex::new(None);

/// bo2 merge: START's map, for the front end to start (`take_pending_zone`).
static PENDING_ZONE: Mutex<Option<String>> = Mutex::new(None);

/// The Zombies map START asked for (`t6:zm_transit`), once.
pub(crate) fn take_pending_zone() -> Option<String> {
    PENDING_ZONE.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take()
}

/// bo2 merge: START's `map` command, run by the console.
pub(crate) fn zombies_exec(mut exec: MessageWriter<frame::UiExecCommand>) {
    if let Some(text) = PENDING_MAP.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take() {
        exec.write(frame::UiExecCommand { text });
    }
}

fn with<R>(f: impl FnOnce(&mut ZmState) -> R) -> R {
    let mut g = STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let st = g.get_or_insert_with(ZmState::default);
    if st.install.is_none() && !PREVIEW.load(Ordering::Relaxed) {
        st.install = Some(find_install());
    }
    f(st)
}

pub(crate) fn state() -> ZmState {
    with(|s| s.clone())
}

pub(crate) fn running() -> bool {
    with(|s| s.running.is_some())
}

/// Look for the Zombies game again (opening the Zombies screens: it may
/// have been built since).
pub(crate) fn refresh() {
    if PREVIEW.load(Ordering::Relaxed) {
        return;
    }
    let found = find_install();
    match &found {
        Ok(i) => diag::info!(Ui, "bo2mp zombies: game at {} (Black Ops maps: {})", i.exe.display(), if i.extra.is_empty() { "off" } else { "on" }),
        Err(e) => diag::warn!(Ui, "bo2mp zombies: {e}"),
    }
    with(|s| s.install = Some(found));
}

/// The preview's stand-in: an install (or the reason there is none), and a
/// game running or not.
pub(crate) fn set_preview(install: Result<Install, String>, running: Option<&str>) {
    PREVIEW.store(true, Ordering::Relaxed);
    let mut g = STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *g = Some(ZmState { install: Some(install), running: running.map(str::to_owned) });
}

fn unquote(v: &str) -> &str {
    let v = v.trim();
    v.strip_prefix('"').and_then(|v| v.strip_suffix('"')).or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\''))).unwrap_or(v)
}

/// The Zombies Declassified folders its `.env` names (`IW4L_T6_EXTRA`,
/// `;`-separated; a commented line does not count), those that exist.
fn extra_folders(env_text: &str) -> Vec<PathBuf> {
    env_text
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.strip_prefix("export ").unwrap_or(l).split_once('='))
        .filter(|(k, _)| k.trim() == "IW4L_T6_EXTRA")
        .flat_map(|(_, v)| unquote(v).split(';').map(|p| PathBuf::from(p.trim())).collect::<Vec<_>>())
        .filter(|p| !p.as_os_str().is_empty() && p.is_dir())
        .collect()
}

/// Where the build script puts the Zombies game: `<project>\zombies\play`,
/// beside this game's `<project>\play`.
fn default_exe() -> Option<PathBuf> {
    let here = std::env::current_exe().ok()?;
    let project = here.parent()?.parent()?;
    Some(project.join("zombies").join("play").join(if cfg!(windows) { "iw4l.exe" } else { "iw4l" }))
}

/// bo2 merge: Zombies plays in this game (one build since the merge);
/// `BO2MP_ZOMBIES_EXTERNAL=1` starts the separate Zombies program instead.
fn in_process() -> bool {
    std::env::var("BO2MP_ZOMBIES_EXTERNAL").map_or(true, |v| v.trim() == "0" || v.trim().is_empty())
}

fn find_install() -> Result<Install, String> {
    if in_process() {
        // This game, its play folder, and its own Zombies Declassified
        // folders (`IW4L_T6_EXTRA` in its .env).
        let exe = std::env::current_exe().unwrap_or_default();
        let dir = std::env::current_dir().unwrap_or_default();
        let extra = asset_transport::t6_extra::extra_roots().to_vec();
        return Ok(Install { exe, dir, extra, shots: None });
    }
    let set = std::env::var("BO2MP_ZOMBIES_EXE").ok().map(|v| unquote(&v).to_owned()).filter(|v| !v.is_empty());
    let exe = match set.clone().map(PathBuf::from).or_else(default_exe) {
        Some(p) => p,
        None => return Err("The Zombies game was not found: set BO2MP_ZOMBIES_EXE in this game's .env.".to_owned()),
    };
    if !exe.is_file() {
        let how = if set.is_some() { "BO2MP_ZOMBIES_EXE in this game's .env names" } else { "It is looked for at" };
        return Err(format!(
            "The Zombies game is not built yet. {how} {}. Run run-bo2.cmd again: it builds Zombies (DaffyDabz/bo2-zombies-rebuild) into the project's zombies folder.",
            exe.display()
        ));
    }
    let dir = exe.parent().map(Path::to_path_buf).unwrap_or_default();
    let extra = std::fs::read_to_string(dir.join(".env")).map(|t| extra_folders(&t)).unwrap_or_default();
    let shots = dir.parent().map(|p| p.join("source").join("docs").join("screenshots")).filter(|p| p.is_dir());
    Ok(Install { exe, dir, extra, shots })
}

/// START: the Zombies game on the chosen map, start and round.
pub(crate) fn start(shell: &Shell) -> Result<(), String> {
    if running() {
        return Err("Zombies is already running.".to_owned());
    }
    refresh();
    let install = state().install.unwrap_or_else(|| Err("The Zombies game was not found.".to_owned()))?;
    let (m, z) = chosen(shell);
    if m.bo1 && install.extra.is_empty() {
        return Err(format!("{} needs the Zombies Declassified map pack (IW4L_T6_EXTRA in the game's .env, set by run-bo2.cmd when it finds the pack).", m.title));
    }
    let round = ROUNDS[shell.zm_round.min(ROUNDS.len() - 1)];
    let spec = launch_spec(m, shell.zm_start, round);
    let line = game_line(m, z);
    if PREVIEW.load(Ordering::Relaxed) {
        with(|s| s.running = Some(line));
        return Ok(());
    }
    if in_process() {
        // As Zombies' own front end starts a game: its start and mode
        // picked, the starting round, then `map t6:<zone>` here.
        let zone = spec.args.get(1).cloned().unwrap_or_default();
        let map = zone.strip_prefix("t6:").unwrap_or(&zone).to_owned();
        if let Some((_, v)) = spec.env.iter().find(|(k, _)| k == "IW4L_T6_START")
            && let Some((loc, mode)) = v.split_once(':')
        {
            frame::set_bo2_zm_start(&map, loc, mode);
        }
        sim::set_t6_party_size(0);
        sim::set_t6_game_settings(if round > 1 { vec![("startround".to_owned(), round as i32)] } else { Vec::new() });
        diag::info!(Ui, "bo2mp zombies: starting {line} here ({zone}, round {round})");
        // (Started as the front end starts a match: the front end is put
        // away and the session swaps the map in, so its screens do not stay
        // over the game.)
        *PENDING_ZONE.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(zone);
        return Ok(());
    }
    let mut cmd = std::process::Command::new(&install.exe);
    cmd.args(&spec.args).current_dir(&install.dir);
    // (Our own IW4L_* settings - our games folder above all, which holds
    // every game - must not reach it: its `.env` names Black Ops II alone,
    // and a set variable wins over its `.env`.)
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("IW4L_") {
            cmd.env_remove(&k);
        }
    }
    cmd.envs(spec.env.iter().map(|(k, v)| (k, v)));
    cmd.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: no console window beside the game.
        cmd.creation_flags(0x0800_0000);
    }
    let child = cmd.spawn().map_err(|e| format!("Zombies did not start ({}): {e}", install.exe.display()))?;
    diag::info!(Ui, "bo2mp zombies: started {} {:?} with {:?} in {} (pid {})", install.exe.display(), spec.args, spec.env, install.dir.display(), child.id());
    *CHILD.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((child, std::time::Instant::now(), install.dir.clone()));
    with(|s| s.running = Some(line));
    Ok(())
}

/// The Zombies game closed: a line for him (`None` while it runs).
fn poll() -> Option<String> {
    if PREVIEW.load(Ordering::Relaxed) {
        return None;
    }
    let mut g = CHILD.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (child, since, dir) = g.as_mut()?;
    let status = match child.try_wait() {
        Ok(None) => return None,
        Ok(Some(status)) => Some(status),
        Err(e) => {
            diag::warn!(Ui, "bo2mp zombies: lost track of the game: {e}");
            None
        }
    };
    let secs = since.elapsed().as_secs_f64();
    let logs = dir.join("iw4l-artifacts").join("logs");
    *g = None;
    drop(g);
    with(|s| s.running = None);
    diag::info!(Ui, "bo2mp zombies: the game closed after {secs:.0} s ({status:?})");
    // (A game that closes at once did not start: say where its log is.)
    Some(match status {
        Some(st) if !st.success() && secs < 30.0 => {
            format!("Zombies closed after {secs:.0} s (exit code {}). Its log is in {}.", st.code().map_or_else(|| "none".to_owned(), |c| c.to_string()), logs.display())
        }
        _ => "Back from Zombies.".to_owned(),
    })
}

/// What was set aside while the Zombies game runs.
#[derive(Default)]
pub(crate) struct Away {
    active: bool,
    winit: Option<bevy::winit::WinitSettings>,
    paused: Vec<Entity>,
}

/// Once a frame: while the Zombies game runs our window is minimised, runs
/// four updates a second and its sounds are paused; when it closes all of
/// that comes back and he is on the Zombies lobby again.
pub(crate) fn zombies_watch(
    mut shell: ResMut<Shell>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut winit: Option<ResMut<bevy::winit::WinitSettings>>,
    sinks: Query<(Entity, &AudioSink)>,
    time: Res<Time>,
    mut away: Local<Away>,
) {
    let preview = PREVIEW.load(Ordering::Relaxed);
    if running() && !away.active {
        away.active = true;
        if !preview {
            if let Ok(mut w) = windows.single_mut() {
                w.set_minimized(true);
            }
            if let Some(ws) = winit.as_deref_mut() {
                away.winit = Some(ws.clone());
                let pace = bevy::winit::UpdateMode::reactive_low_power(std::time::Duration::from_millis(250));
                *ws = bevy::winit::WinitSettings { focused_mode: pace, unfocused_mode: pace };
            }
        }
    }
    if !away.active || preview {
        return;
    }
    // (Each frame: a track the music player starts meanwhile is paused too.)
    for (e, sink) in &sinks {
        if !sink.is_paused() {
            sink.pause();
            if !away.paused.contains(&e) {
                away.paused.push(e);
            }
        }
    }
    let Some(line) = poll() else { return };
    if let (Some(saved), Some(ws)) = (away.winit.take(), winit.as_deref_mut()) {
        *ws = saved;
    }
    if let Ok(mut w) = windows.single_mut() {
        w.set_minimized(false);
        w.focused = true;
    }
    for (e, sink) in &sinks {
        if away.paused.contains(&e) {
            sink.play();
        }
    }
    away.paused.clear();
    away.active = false;
    shell.screen = Screen::Zombies;
    shell.focus = 0;
    let now = time.elapsed_secs_f64();
    shell.note(&line, now);
    shell.now = now;
}

// ---------------------------------------------------------------------------
// Pictures.

/// The pictures to try for a map's start, best first: Black Ops II's own
/// loading and lobby pictures for it (read from his install's Zombies
/// zones), any of the map's, then the Zombies rebuild's screenshot.
pub(crate) fn pictures(m: &ZmMap, start: usize) -> Vec<String> {
    let s = &m.starts[start.min(m.starts.len() - 1)];
    let group = match s.mode {
        "zclassic" => "zclassic",
        "zgrief" | "zcleansed" => "zencounter",
        _ => "zsurvival",
    };
    let mut out = Vec::new();
    if !s.loc.is_empty() {
        out.push(format!("loadscreen_{}_{}_{}", m.map, s.mode, s.loc));
        out.push(format!("menu_{}_{}_{}", m.map, group, s.loc));
        out.push(format!("zmlike:loadscreen_{}_{}", m.map, s.mode));
        out.push(format!("zmlike:menu_{}_{}", m.map, group));
    }
    out.push(format!("zmlike:loadscreen_{}", m.map));
    out.push(format!("zmshot:{}", m.shot));
    out
}

/// A picture name the Zombies screens ask for.
pub(crate) fn is_zm_picture(name: &str) -> bool {
    name.starts_with("zmshot:") || name.starts_with("zmlike:") || name.starts_with("loadscreen_zm_") || name.starts_with("menu_zm_")
}

/// Black Ops II's Zombies pictures are still being read (the first time
/// from his install's zones, after that from a cache).
pub(crate) fn pictures_busy() -> bool {
    READING.load(Ordering::Relaxed)
}

static READING: AtomicBool = AtomicBool::new(false);

type Rgba = (u32, u32, Vec<u8>);

/// A picture for the Zombies screens (read off the main thread; `None`
/// until it is in, or when there is none).
pub(crate) fn picture(name: &str) -> Option<Arc<Image>> {
    if let Some(stem) = name.strip_prefix("zmshot:") {
        let path = match &state().install {
            Some(Ok(Install { shots: Some(dir), .. })) => dir.join(format!("{stem}.jpg")),
            _ => return None,
        };
        return crate::foreign_maps::picture_async(name, true, move || read_jpg(&path));
    }
    let want = name.to_owned();
    crate::foreign_maps::picture_async(name, true, move || {
        let set = zm_pictures();
        let found = match want.strip_prefix("zmlike:") {
            Some(prefix) => set.iter().filter(|(k, _)| k.starts_with(prefix)).min_by(|a, b| a.0.cmp(b.0)).map(|(_, v)| v.clone()),
            None => set.get(&want).cloned(),
        };
        found.map(crate::foreign_maps::upscale_map_shot)
    })
}

fn read_jpg(path: &Path) -> Option<Rgba> {
    let bytes = std::fs::read(path).ok()?;
    let img = Image::from_buffer(
        &bytes,
        bevy::image::ImageType::Extension("jpg"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        bevy::asset::RenderAssetUsages::default(),
    )
    .ok()?;
    let (w, h) = (img.width(), img.height());
    let data = img.data?;
    (data.len() == (w * h * 4) as usize).then_some((w, h, data))
}

const CACHE: &str = "iw4l-artifacts/cache/zm-pictures";

/// Every `menu_zm_*` / `loadscreen_zm_*` picture, read once a run: from
/// the cache when it was made with the same map pack, else from his Black
/// Ops II install's Zombies zones (then cached).
fn zm_pictures() -> &'static std::collections::HashMap<String, Rgba> {
    static SET: OnceLock<std::collections::HashMap<String, Rgba>> = OnceLock::new();
    SET.get_or_init(|| {
        READING.store(true, Ordering::Relaxed);
        let started = std::time::Instant::now();
        let extra: Vec<PathBuf> = match &state().install {
            Some(Ok(i)) => i.extra.iter().filter_map(|d| find_file(d, "mod_load.ff", 6)).collect(),
            _ => Vec::new(),
        };
        let stamp = format!("extra={}", extra.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(";"));
        let dir = Path::new(CACHE);
        let cached = std::fs::read_to_string(dir.join("index.txt")).ok().filter(|t| t.lines().next() == Some(stamp.as_str()));
        let set: std::collections::HashMap<String, Rgba> = match cached {
            Some(index) => index
                .lines()
                .skip(1)
                .filter_map(|n| {
                    let img = image::open(dir.join(format!("{n}.png"))).ok()?.to_rgba8();
                    Some((n.to_owned(), (img.width(), img.height(), img.into_raw())))
                })
                .collect(),
            None => {
                let zone_all = asset_transport::games_root_from_env()
                    .ok()
                    .and_then(|root| asset_transport::find_zone_file(&root, "t6:patch_mp").ok())
                    .and_then(|z| z.path.parent().map(Path::to_path_buf));
                let Some(zone_all) = zone_all else {
                    diag::warn!(Ui, "bo2mp zombies: no Black Ops II zone folder for the Zombies pictures");
                    READING.store(false, Ordering::Relaxed);
                    return Default::default();
                };
                let (found, report) = assets::load_t6_zm_pictures(&zone_all, &extra);
                for line in &report {
                    diag::info!(Ui, "bo2mp zombies: {line}");
                }
                let set: std::collections::HashMap<String, Rgba> = found
                    .into_iter()
                    .filter_map(|(n, img)| {
                        let (w, h) = (img.width(), img.height());
                        let data = img.data?;
                        (data.len() == (w * h * 4) as usize).then_some((n, (w, h, data)))
                    })
                    .collect();
                let _ = std::fs::create_dir_all(dir);
                let mut index = vec![stamp];
                for (n, (w, h, data)) in &set {
                    if image::RgbaImage::from_raw(*w, *h, data.clone()).is_some_and(|i| i.save(dir.join(format!("{n}.png"))).is_ok()) {
                        index.push(n.clone());
                    }
                }
                let _ = std::fs::write(dir.join("index.txt"), index.join("\n"));
                set
            }
        };
        diag::info!(Ui, "bo2mp zombies: {} Black Ops II Zombies pictures in {:.1} s", set.len(), started.elapsed().as_secs_f32());
        READING.store(false, Ordering::Relaxed);
        set
    })
}

/// The first file named `name` (any case) under `dir`, `depth` folders down.
fn find_file(dir: &Path, name: &str, depth: usize) -> Option<PathBuf> {
    let mut subdirs = Vec::new();
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            subdirs.push(p);
        } else if p.file_name().is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name)) {
            return Some(p);
        }
    }
    if depth == 0 {
        return None;
    }
    subdirs.sort();
    subdirs.into_iter().find_map(|d| find_file(&d, name, depth - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(map_key: &str, start: usize, round: u32) -> ZmLaunch {
        launch_spec(map(map_key), start, round)
    }

    fn env(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect()
    }

    #[test]
    fn story_mode_goes_straight_to_the_map() {
        let s = spec("zm_transit", 0, 1);
        assert_eq!(s.args, vec!["map", "t6:zm_transit"]);
        assert_eq!(s.env, env(&[("IW4L_T6_START", "transit:zclassic")]));
    }

    #[test]
    fn a_start_and_mode_are_the_start_variable() {
        assert_eq!(spec("zm_transit", 1, 1).env, env(&[("IW4L_T6_START", "transit:zstandard")]));
        assert_eq!(spec("zm_transit", 3, 1).env, env(&[("IW4L_T6_START", "town:zgrief")]));
        assert_eq!(spec("zm_transit", 4, 1).env, env(&[("IW4L_T6_START", "farm:zstandard")]));
        assert_eq!(spec("zm_prison", 1, 1).env, env(&[("IW4L_T6_START", "cellblock:zgrief")]));
        assert_eq!(spec("zm_buried", 2, 1).env, env(&[("IW4L_T6_START", "street:zcleansed")]));
        assert_eq!(spec("zm_nuked", 0, 1).env, env(&[("IW4L_T6_START", "nuked:zstandard")]));
        assert_eq!(spec("zm_highrise", 0, 1).env, env(&[("IW4L_T6_START", "rooftop:zclassic")]));
        assert_eq!(spec("zm_tomb", 0, 1).env, env(&[("IW4L_T6_START", "tomb:zclassic")]));
    }

    #[test]
    fn turned_at_the_diner_loads_its_own_zone() {
        let s = spec("zm_transit", 6, 1);
        assert_eq!(s.args, vec!["map", "t6:zm_transit_dr"]);
        assert_eq!(s.env, env(&[("IW4L_T6_START", "diner:zcleansed")]));
    }

    #[test]
    fn a_later_starting_round_is_its_variable() {
        let s = spec("zm_buried", 1, 15);
        assert_eq!(s.args, vec!["map", "t6:zm_buried"]);
        assert_eq!(s.env, env(&[("IW4L_T6_START", "street:zgrief"), ("IW4L_T6_START_ROUND", "15")]));
    }

    #[test]
    fn black_ops_maps_start_at_their_own_start() {
        let s = spec("zm_moon", 0, 1);
        assert_eq!(s.args, vec!["map", "t6:zm_moon"]);
        assert!(s.env.is_empty());
        assert_eq!(spec("zm_asylum", 0, 10).env, env(&[("IW4L_T6_START_ROUND", "10")]));
    }

    #[test]
    fn a_start_past_the_list_is_the_last() {
        assert_eq!(spec("zm_tomb", 9, 1), spec("zm_tomb", 0, 1));
    }

    #[test]
    fn every_map_has_a_start_and_the_readme_count() {
        assert!(MAPS.iter().all(|m| !m.starts.is_empty()));
        assert_eq!(MAPS.iter().filter(|m| !m.bo1).count(), 6);
        assert_eq!(MAPS.iter().filter(|m| m.bo1).count(), 9);
        assert_eq!(map("zm_transit").starts.len(), 7);
        assert_eq!(pages(false).len(), 1);
        assert_eq!(pages(true)[1].1.len(), 9);
    }

    #[test]
    fn the_pack_folder_is_read_from_its_env() {
        let dir = std::env::temp_dir().join(format!("bo2mp-zm-env-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let d = dir.display().to_string().replace('\\', "/");
        assert_eq!(extra_folders(&format!("IW4L_GAMES=\"C:/x\"\nIW4L_T6_EXTRA=\"{d}\"\n")), vec![PathBuf::from(&d)]);
        assert!(extra_folders(&format!("# IW4L_T6_EXTRA=\"{d}\"\n")).is_empty());
        assert!(extra_folders("IW4L_T6_EXTRA=\"Z:/no/such/folder\"\n").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pictures_end_with_the_screenshot() {
        let p = pictures(map("zm_prison"), 1);
        assert_eq!(p[0], "loadscreen_zm_prison_zgrief_cellblock");
        assert_eq!(p[1], "menu_zm_prison_zencounter_cellblock");
        assert_eq!(p.last().unwrap(), "zmshot:zm_prison");
        assert!(p.iter().all(|n| is_zm_picture(n)));
        assert_eq!(pictures(map("zm_moon"), 0), vec!["zmlike:loadscreen_zm_moon", "zmshot:zm_bo1_moon"]);
    }
}
