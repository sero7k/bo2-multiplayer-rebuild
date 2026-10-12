//! The music director's Bevy side: builds the track table and playlists from
//! `music.cfg`, the game tracks and the user's music folders; works out the
//! context each frame (menu, lobby, match, or silence while loading); carries
//! out the [`Core`]'s actions (prepare, start, stop); sets every music voice's
//! sink volume from its fade, its context's level, `music_volume` and
//! `master_volume`; and keeps `<artifacts>/audio_manifest.csv` current.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use asset_core::AssetNamespace;
use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, PlaybackSettings, Volume};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, TaskPool, futures_lite::future};
use frame::{AppScreen, ClientSet, GameSettings, LaunchIdentity, MenuMusicScreen, UiPlayMusic, UiStopMusic};

use super::config::{
    self, Entry, GAME_TRACKS, MusicConfig, game_label, is_music_file, scan_dir, unreadable_game,
    unwired_game,
};
use super::logic::{Action, Core, MusicContext, Playlist, TrackId};
use super::manifest::{self, ManifestRow};
use super::stream::{Probe, StreamedMusic, probe};
use crate::clip_store::{ClipKey, ClipStore, clip_keys_for_alias};
use crate::frontend::FrontendAudio;
use crate::pcm::PcmAudio;

/// The biggest music file read (they are held in memory while they play).
const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
/// A voice whose sink runs dry this soon after starting never really played.
/// How long a new voice may go without its entity or sink before it counts
/// as gone.
const SINK_GRACE_S: f64 = 5.0;
const DEAD_ON_ARRIVAL_S: f64 = 1.0;
/// The manifest is rewritten at most this often.
const MANIFEST_EVERY_S: f64 = 2.0;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Source {
    Game(AssetNamespace, String),
    File(PathBuf),
}

#[derive(Clone, Debug)]
struct Track {
    source: Source,
    intended: &'static str,
    origin: String,
    format: String,
    length: Option<f64>,
    playable: Option<bool>,
    reason: String,
}

impl Track {
    fn label(&self) -> String {
        match &self.source {
            Source::Game(ns, alias) => format!("{} `{alias}`", game_label(*ns)),
            Source::File(path) => format!(
                "`{}`",
                path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned())
            ),
        }
    }
}

enum Prepared {
    Pcm(Handle<PcmAudio>),
    Stream(Handle<StreamedMusic>),
}

enum Preparing {
    File {
        track: TrackId,
        task: Task<Result<(Arc<[u8]>, Probe), String>>,
    },
    Game {
        track: TrackId,
        key: ClipKey,
    },
}

/// A music voice (its director voice id, for inspection).
#[derive(Component, Debug)]
pub struct MusicVoice {
    pub id: u64,
}

struct LiveVoice {
    entity: Entity,
    track: TrackId,
    started: f64,
    had_sink: bool,
}

#[derive(Resource)]
pub(crate) struct MusicDirector {
    core: Core,
    cfg: MusicConfig,
    tracks: Vec<Track>,
    by_source: HashMap<Source, TrackId>,
    extras: Vec<ManifestRow>,
    clock: f64,
    voices: HashMap<u64, LiveVoice>,
    prepared: HashMap<TrackId, Prepared>,
    preparing: Option<Preparing>,
    probes: Option<Task<Vec<(TrackId, Result<Probe, String>)>>>,
    game_store: Option<(Arc<asset_audio::SoundCatalog>, ClipStore)>,
    bank_checked: bool,
    manifest_path: Option<PathBuf>,
    manifest_dirty: bool,
    manifest_written_at: f64,
}

impl MusicDirector {
    fn add_track(&mut self, source: Source, intended: &'static str) -> TrackId {
        if let Some(&id) = self.by_source.get(&source) {
            return id;
        }
        let (origin, format) = match &source {
            Source::Game(..) => (String::new(), String::new()),
            Source::File(path) => (
                path.display().to_string(),
                path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default(),
            ),
        };
        let id = self.tracks.len();
        self.tracks.push(Track {
            source: source.clone(),
            intended,
            origin,
            format,
            length: None,
            playable: None,
            reason: "not checked yet".to_owned(),
        });
        self.by_source.insert(source, id);
        id
    }

    fn mark(&mut self, track: TrackId, playable: bool, reason: impl Into<String>) {
        if let Some(t) = self.tracks.get_mut(track) {
            t.playable = Some(playable);
            t.reason = reason.into();
            self.manifest_dirty = true;
        }
    }

    fn fail(&mut self, track: TrackId, reason: &str) {
        let label = self.tracks.get(track).map(Track::label).unwrap_or_default();
        diag::warn!(Audio, "audio: music: {label} is unplayable ({reason}); skipped");
        self.mark(track, false, reason);
        self.core.failed(track);
        self.prepared.remove(&track);
    }

    fn rows(&self) -> Vec<ManifestRow> {
        let mut rows: Vec<ManifestRow> = self
            .tracks
            .iter()
            .enumerate()
            .map(|(id, t)| {
                let lists: Vec<&str> = MusicContext::PLAYING
                    .iter()
                    .filter(|c| self.core.playlist(**c).is_some_and(|p| p.contains(id)))
                    .map(|c| c.label())
                    .collect();
                let (game, alias, source) = match &t.source {
                    Source::Game(ns, alias) => (game_label(*ns).to_owned(), alias.clone(), t.origin.clone()),
                    Source::File(_) => ("user file".to_owned(), String::new(), t.origin.clone()),
                };
                ManifestRow {
                    game,
                    source,
                    alias,
                    format: t.format.clone(),
                    duration_s: t.length,
                    intended_use: t.intended.to_owned(),
                    playable: match t.playable {
                        Some(true) => "yes",
                        Some(false) => "no",
                        None => "unknown",
                    }
                    .to_owned(),
                    reason: t.reason.clone(),
                    playlists: if self.cfg.enabled { lists.join("|") } else { String::new() },
                }
            })
            .collect();
        rows.extend(self.extras.iter().cloned());
        rows
    }

    fn write_manifest(&mut self, why: &str) {
        self.manifest_dirty = false;
        self.manifest_written_at = self.clock;
        let Some(path) = self.manifest_path.clone() else {
            return;
        };
        let rows = self.rows();
        match manifest::write(&path, &rows) {
            Ok(()) => diag::info!(
                Audio,
                "audio: music: manifest {} ({why}): {}",
                path.display(),
                manifest::summary(&rows)
            ),
            Err(error) => diag::warn!(Audio, "audio: music: manifest {} not written ({error})", path.display()),
        }
    }
}

/// The context for this frame.
pub(crate) fn resolve_context(
    enabled: bool,
    screen: Option<AppScreen>,
    loading: bool,
    menu: MenuMusicScreen,
    match_music: bool,
) -> MusicContext {
    if !enabled {
        return MusicContext::Silent;
    }
    match screen {
        Some(AppScreen::InGame | AppScreen::ClassSelect) if match_music => MusicContext::Match,
        Some(AppScreen::MainMenu) if !loading => match menu {
            MenuMusicScreen::Main => MusicContext::Menu,
            MenuMusicScreen::Lobby => MusicContext::Lobby,
        },
        _ => MusicContext::Silent,
    }
}

fn games_folder_rows(roots: &[PathBuf]) -> Vec<ManifestRow> {
    let mut rows = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut visit = |dir: &Path, rows: &mut Vec<ManifestRow>| {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if !path.is_dir() || !seen.insert(path.clone()) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(game) = unreadable_game(&name) {
                rows.push(ManifestRow {
                    game: game.to_owned(),
                    source: path.display().to_string(),
                    intended_use: "other".to_owned(),
                    playable: "no".to_owned(),
                    reason: "not readable by this build (unsupported game format)".to_owned(),
                    ..ManifestRow::default()
                });
            } else if let Some(game) = unwired_game(&name) {
                rows.push(ManifestRow {
                    game: game.to_owned(),
                    source: path.display().to_string(),
                    intended_use: "other".to_owned(),
                    playable: "no".to_owned(),
                    reason: "game installed, but the music director has no music loader for it".to_owned(),
                    ..ManifestRow::default()
                });
            }
        }
    };
    for root in roots {
        visit(root, &mut rows);
        // One level down too (`<root>/<store>/<game>`).
        if let Ok(read) = std::fs::read_dir(root) {
            for entry in read.flatten() {
                let p = entry.path();
                if p.is_dir()
                    && unreadable_game(&entry.file_name().to_string_lossy()).is_none()
                    && unwired_game(&entry.file_name().to_string_lossy()).is_none()
                {
                    visit(&p, &mut rows);
                }
            }
        }
    }
    rows
}

fn build_director(identity: Option<&LaunchIdentity>) -> MusicDirector {
    let exe_dir = config::exe_dir();
    let root = config::music_root(&exe_dir);
    for sub in ["menu", "lobby", "match"] {
        let dir = root.join(sub);
        if !dir.is_dir() {
            match std::fs::create_dir_all(&dir) {
                Ok(()) => diag::info!(Audio, "audio: music: made {} for your music", dir.display()),
                Err(error) => diag::info!(Audio, "audio: music: no {} ({error})", dir.display()),
            }
        }
    }
    let (cfg_path, exists) = config::config_path(&exe_dir, &root);
    let mut cfg = if exists {
        match std::fs::read_to_string(&cfg_path) {
            Ok(text) => {
                let mut cfg = MusicConfig::parse(&text, &exe_dir);
                cfg.source = Some(cfg_path.clone());
                cfg
            }
            Err(error) => {
                diag::warn!(Audio, "audio: music: {} not read ({error}); defaults", cfg_path.display());
                MusicConfig::default()
            }
        }
    } else {
        match std::fs::write(&cfg_path, config::TEMPLATE) {
            Ok(()) => diag::info!(Audio, "audio: music: wrote a template {}", cfg_path.display()),
            Err(error) => diag::info!(Audio, "audio: music: no {} ({error}); defaults", cfg_path.display()),
        }
        MusicConfig::default()
    };
    for warning in cfg.warnings.drain(..) {
        diag::warn!(Audio, "audio: music: music.cfg {warning}");
    }

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9e37_79b9_7f4a_7c15, |d| d.as_nanos() as u64);
    let mut director = MusicDirector {
        core: Core::new(cfg.timings, Default::default(), seed),
        cfg: cfg.clone(),
        tracks: Vec::new(),
        by_source: HashMap::new(),
        extras: Vec::new(),
        clock: 0.0,
        voices: HashMap::new(),
        prepared: HashMap::new(),
        preparing: None,
        probes: None,
        game_store: None,
        bank_checked: false,
        manifest_path: identity
            .filter(|i| !i.artifacts.as_os_str().is_empty())
            .map(|i| i.artifacts.join("audio_manifest.csv")),
        manifest_dirty: true,
        manifest_written_at: f64::NEG_INFINITY,
    };

    // The defaults: the games' menu tracks, then each folder's files.
    let game: Vec<TrackId> = GAME_TRACKS
        .iter()
        .map(|(ns, alias, _)| director.add_track(Source::Game(*ns, (*alias).to_owned()), "menu"))
        .collect();
    let folder = |director: &mut MusicDirector, sub: &str, intended: &'static str| -> Vec<TrackId> {
        let dir = root.join(sub);
        let (music, other) = scan_dir(&dir);
        for path in other {
            director.extras.push(ManifestRow {
                game: "user file".to_owned(),
                source: path.display().to_string(),
                format: path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default(),
                intended_use: intended.to_owned(),
                playable: "no".to_owned(),
                reason: "not a music file this build reads (.mp3 .ogg .wav .flac)".to_owned(),
                ..ManifestRow::default()
            });
        }
        music
            .into_iter()
            .map(|p| director.add_track(Source::File(p), intended))
            .collect()
    };
    let menu_files = folder(&mut director, "menu", "menu");
    let lobby_files = folder(&mut director, "lobby", "lobby");
    let match_files = folder(&mut director, "match", "ambience");
    let menu_default: Vec<TrackId> = game.iter().chain(&menu_files).copied().collect();
    let lobby_default = if lobby_files.is_empty() { menu_default.clone() } else { lobby_files };
    // (No menu tracks in a match: only his own match music, if any.)
    let match_default = match_files;
    let defaults = [menu_default, lobby_default, match_default];

    let mut lists: [Vec<TrackId>; 3] = Default::default();
    for slot in 0..3 {
        let Some(entries) = cfg.playlists[slot].clone() else {
            lists[slot] = defaults[slot].clone();
            continue;
        };
        for entry in entries {
            match entry {
                Entry::Defaults => lists[slot].extend(&defaults[slot]),
                Entry::None => {}
                Entry::Game(ns, alias) => lists[slot].push(director.add_track(Source::Game(ns, alias), "custom")),
                Entry::Path(path) if path.is_dir() => {
                    let (music, _) = scan_dir(&path);
                    for p in music {
                        lists[slot].push(director.add_track(Source::File(p), "custom"));
                    }
                }
                Entry::Path(path) => {
                    let id = director.add_track(Source::File(path.clone()), "custom");
                    if !path.is_file() {
                        director.mark(id, false, "file not found");
                    } else if !is_music_file(&path) {
                        director.mark(id, false, "not a music file this build reads (.mp3 .ogg .wav .flac)");
                    }
                    lists[slot].push(id);
                }
            }
        }
    }
    let shuffle = cfg.shuffle;
    let [menu, lobby, matc] = lists;
    director.core = Core::new(
        cfg.timings,
        [
            Playlist::new(menu, shuffle),
            Playlist::new(lobby, shuffle),
            Playlist::new(matc, shuffle),
        ],
        seed,
    );
    for id in 0..director.tracks.len() {
        if director.tracks[id].playable == Some(false) {
            director.core.failed(id);
        }
    }

    // Game folders this build cannot read, for the manifest.
    let mut roots: Vec<PathBuf> = identity
        .map(|i| i.games_root.clone())
        .filter(|p| !p.as_os_str().is_empty())
        .into_iter()
        .collect();
    if let Some(voice) = asset_audio::voice_games_root() {
        roots.push(voice.0);
    }
    director.extras.extend(games_folder_rows(&roots));

    // Probe every file in the background: format, length, playable.
    let files: Vec<(TrackId, PathBuf)> = director
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.playable.is_none())
        .filter_map(|(id, t)| match &t.source {
            Source::File(p) => Some((id, p.clone())),
            Source::Game(..) => None,
        })
        .collect();
    if !files.is_empty() {
        let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
        director.probes = Some(pool.spawn(async move {
            files
                .into_iter()
                .map(|(id, path)| (id, read_and_probe(&path).map(|(_, p)| p)))
                .collect()
        }));
    }

    let count = |c: MusicContext| director.core.playlist(c).map_or(0, |p| p.tracks().len());
    diag::info!(
        Audio,
        "audio: music: director up — config {}, music folder {}, playlists menu {} / lobby {} / match {} tracks, shuffle {}, fades in {:.1} s / out {:.1} s / cross {:.1} s{}",
        cfg.source.as_ref().map_or_else(|| "defaults".to_owned(), |p| p.display().to_string()),
        root.display(),
        count(MusicContext::Menu),
        count(MusicContext::Lobby),
        count(MusicContext::Match),
        if shuffle { "on" } else { "off" },
        cfg.timings.fade_in,
        cfg.timings.fade_out,
        cfg.timings.crossfade,
        if cfg.enabled { "" } else { " — DISABLED by music.cfg" },
    );
    director
}

fn read_and_probe(path: &Path) -> Result<(Arc<[u8]>, Probe), String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("not readable ({e})"))?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(format!("too big ({} MB)", meta.len() / (1024 * 1024)));
    }
    let bytes: Arc<[u8]> = std::fs::read(path).map_err(|e| format!("not readable ({e})"))?.into();
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase());
    let probed = probe(Arc::clone(&bytes), ext.as_deref())?;
    Ok((bytes, probed))
}

/// The front end's bank was resident: check every game track against it and
/// list the other music it holds.
fn check_bank(director: &mut MusicDirector, frontend: &FrontendAudio) {
    director.bank_checked = true;
    let bank = &frontend.bank;
    for id in 0..director.tracks.len() {
        let Source::Game(ns, alias) = director.tracks[id].source.clone() else {
            continue;
        };
        let keys = clip_keys_for_alias(bank, ns, &alias);
        let zone = bank
            .index_in(ns, &alias)
            .map(|i| bank.zone_of_alias(i).as_str().to_owned())
            .unwrap_or_default();
        let Some(key) = keys.into_iter().next() else {
            let reason = if bank.index_in(ns, &alias).is_some() {
                "alias has no sound file"
            } else {
                "not installed (alias not in the front end's sound bank)"
            };
            diag::info!(Audio, "audio: music: {} — {reason}; left out", director.tracks[id].label());
            director.mark(id, false, reason);
            director.core.failed(id);
            continue;
        };
        let (origin, format) = describe_key(bank, &key);
        let t = &mut director.tracks[id];
        t.origin = if zone.is_empty() { origin } else { format!("{zone}: {origin}") };
        t.format = format;
        t.reason = "found in the front end's sound bank; not decoded yet".to_owned();
        director.manifest_dirty = true;
    }
    // Every other music alias the bank holds, for the manifest.
    for (i, sound) in bank.sounds.iter().enumerate() {
        let lower = sound.name.to_ascii_lowercase();
        if !(lower.starts_with("mus_") || lower.starts_with("music_")) {
            continue;
        }
        let ns = bank.namespace_of_alias(i);
        if director.by_source.contains_key(&Source::Game(ns, sound.name.clone())) {
            continue;
        }
        let intended = if lower.contains("frontend") || lower.contains("mainmenu") || lower.contains("menu") {
            "menu"
        } else if lower.contains("lobby") {
            "lobby"
        } else if lower.contains("amb") {
            "ambience"
        } else {
            "other"
        };
        let key = clip_keys_for_alias(bank, ns, &sound.name).into_iter().next();
        let (origin, format) = key.as_ref().map(|k| describe_key(bank, k)).unwrap_or_default();
        director.extras.push(ManifestRow {
            game: game_label(ns).to_owned(),
            source: format!("{}: {origin}", sound.zone.as_str()),
            alias: sound.name.clone(),
            format,
            intended_use: intended.to_owned(),
            playable: if key.is_some() { "unknown" } else { "no" }.to_owned(),
            reason: if key.is_some() {
                format!("in the front end's bank, not in a playlist (add `{}:{}` to music.cfg)", ns.as_str(), sound.name)
            } else {
                "alias has no sound file".to_owned()
            },
            ..ManifestRow::default()
        });
    }
    director.manifest_dirty = true;
}

fn describe_key(bank: &asset_audio::SoundCatalog, key: &ClipKey) -> (String, String) {
    match key {
        ClipKey::Loaded(idx) => {
            let name = bank.loaded.get(*idx).map(|l| l.name.clone()).unwrap_or_default();
            let ext = Path::new(&name).extension().map(|e| e.to_string_lossy().to_ascii_lowercase());
            (format!("loaded {name}"), format!("loaded {}", ext.unwrap_or_else(|| "pcm".into())))
        }
        ClipKey::Streamed { dir, name, .. } => {
            let ext = Path::new(name).extension().map(|e| e.to_string_lossy().to_ascii_lowercase());
            (format!("streamed {dir}/{name}"), format!("streamed {}", ext.unwrap_or_default()))
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct ContextInputs<'w> {
    screen: Option<Res<'w, AppScreen>>,
    menu: Option<Res<'w, MenuMusicScreen>>,
    settings: Option<Res<'w, GameSettings>>,
    request: Option<Res<'w, assets::MatchLoadRequest>>,
    accepted: Option<Res<'w, assets::MatchLoadAccepted>>,
    busy: Option<Res<'w, assets::MatchLoadBusy>>,
}

#[derive(SystemParam)]
pub(crate) struct MusicAssets<'w> {
    pcm: ResMut<'w, Assets<PcmAudio>>,
    streams: ResMut<'w, Assets<StreamedMusic>>,
}

pub(crate) fn init_director(
    director: Option<Res<MusicDirector>>,
    identity: Option<Res<LaunchIdentity>>,
    mut commands: Commands,
) {
    if director.is_some() {
        return;
    }
    let mut built = build_director(identity.as_deref());
    built.write_manifest("startup");
    commands.insert_resource(built);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_director(
    mut director: Option<ResMut<MusicDirector>>,
    time: Res<Time<Real>>,
    inputs: ContextInputs,
    frontend: Option<Res<FrontendAudio>>,
    mut assets: MusicAssets,
    mut play_bed: MessageReader<UiPlayMusic>,
    mut stop_bed: MessageReader<UiStopMusic>,
    mut sinks: Query<Option<&mut AudioSink>, With<MusicVoice>>,
    mut commands: Commands,
) {
    let Some(director) = director.as_deref_mut() else {
        play_bed.clear();
        stop_bed.clear();
        return;
    };
    director.clock += time.delta_secs_f64();
    let now = director.clock;

    if !director.bank_checked
        && let Some(frontend) = frontend.as_deref()
    {
        check_bank(director, frontend);
    }

    // A script menu's music bed joins the menu playlist instead of playing
    // over the director; its stop is the director's to decide.
    stop_bed.clear();
    for bed in play_bed.read() {
        if bed.alias.is_empty() {
            continue;
        }
        let ns = frontend
            .as_deref()
            .and_then(|f| {
                [AssetNamespace::Iw4, AssetNamespace::T6, AssetNamespace::T5, AssetNamespace::Iw5]
                    .into_iter()
                    .find(|ns| f.bank.index_in(*ns, &bed.alias).is_some())
            })
            .unwrap_or(AssetNamespace::Iw4);
        let id = director.add_track(Source::Game(ns, bed.alias.clone()), "menu");
        if director.core.add_to_playlist(MusicContext::Menu, id) {
            director.manifest_dirty = true;
            diag::info!(Audio, "audio: music: menu bed `{}` joins the menu playlist", bed.alias);
        }
    }

    // The file probes from startup.
    if let Some(task) = director.probes.as_mut()
        && let Some(results) = future::block_on(future::poll_once(task))
    {
        director.probes = None;
        for (id, result) in results {
            match result {
                Ok(p) => {
                    let t = &mut director.tracks[id];
                    t.format = format!("{} ({} ch, {} Hz)", p.codec, p.channels, p.rate);
                    t.length = p.length;
                    diag::info!(
                        Audio,
                        "audio: music: user track {} — {}, {}",
                        t.label(),
                        t.format,
                        p.length.map_or_else(|| "length unknown".to_owned(), |l| format!("{l:.1} s"))
                    );
                    director.mark(id, true, "decodes");
                }
                Err(reason) => director.fail(id, &reason),
            }
        }
    }

    // The context.
    let settings = inputs.settings.as_deref().cloned().unwrap_or_default();
    let loading = inputs.request.is_some()
        || inputs.accepted.is_some()
        || inputs.busy.as_deref().is_some_and(|b| b.0);
    let context = resolve_context(
        director.cfg.enabled,
        inputs.screen.as_deref().copied(),
        loading,
        inputs.menu.as_deref().copied().unwrap_or_default(),
        settings.match_music && director.cfg.levels[2] > 0.0,
    );
    let was = director.core.context();
    if context != was {
        diag::info!(Audio, "audio: music: context {} -> {}", was.label(), context.label());
        director.core.set_context(context, now);
    }

    poll_preparing(director, &mut assets);

    for action in director.core.tick(now) {
        match action {
            Action::Prepare { track, context } => start_preparing(director, frontend.as_deref(), track, context),
            Action::Start { voice, track, context, fade } => {
                let Some(prepared) = director.prepared.remove(&track) else {
                    director.fail(track, "prepared clip went missing");
                    continue;
                };
                let settings = PlaybackSettings::ONCE.with_volume(Volume::Linear(0.0));
                let entity = match prepared {
                    Prepared::Pcm(handle) => commands.spawn((AudioPlayer(handle), settings, MusicVoice { id: voice })).id(),
                    Prepared::Stream(handle) => {
                        commands.spawn((AudioPlayer(handle), settings, MusicVoice { id: voice })).id()
                    }
                };
                director.voices.insert(voice, LiveVoice { entity, track, started: now, had_sink: false });
                let t = &director.tracks[track];
                diag::info!(
                    Audio,
                    "audio: music: {} plays {} ({}, fade-in {fade:.1} s)",
                    context.label(),
                    t.label(),
                    t.length.map_or_else(|| "length unknown".to_owned(), |l| format!("{l:.0} s")),
                );
            }
            Action::Stop { voice, track } => {
                if let Some(live) = director.voices.remove(&voice) {
                    commands.entity(live.entity).try_despawn();
                }
                diag::info!(Audio, "audio: music: {} faded out", director.tracks[track].label());
            }
            Action::TimedOut { track } => {
                director.fail(track, "did not prepare in time");
                director.preparing = None;
            }
            Action::Empty { context } => diag::info!(
                Audio,
                "audio: music: the {} playlist has nothing playable — silent",
                context.label()
            ),
        }
    }

    // Volumes; voices whose sink ran dry.
    let master = settings.master_volume.clamp(0.0, 1.0);
    let music = settings.music_volume.clamp(0.0, 1.0);
    let mut ended = Vec::new();
    for (&id, live) in &mut director.voices {
        let Ok(sink) = sinks.get_mut(live.entity) else {
            // (Spawned this frame: its entity only exists once the
            // commands run, so it is not missing yet.)
            if !live.had_sink && now - live.started < SINK_GRACE_S {
                continue;
            }
            // Despawned behind the director's back (a teardown).
            ended.push((id, live.track, now - live.started));
            continue;
        };
        let Some(mut sink) = sink else {
            continue;
        };
        live.had_sink = true;
        if sink.empty() {
            ended.push((id, live.track, now - live.started));
            continue;
        }
        let voice_ctx = director
            .core
            .voices()
            .iter()
            .find(|v| v.id == id)
            .map_or(MusicContext::Silent, |v| v.context);
        let gain = director.core.gain(id, now) * director.cfg.level(voice_ctx) * music * master;
        if (sink.volume().to_linear() - gain).abs() > 1e-4 {
            sink.set_volume(Volume::Linear(gain));
        }
    }
    for (id, track, played) in ended {
        if let Some(live) = director.voices.remove(&id) {
            commands.entity(live.entity).try_despawn();
        }
        director.core.voice_ended(id);
        if played < DEAD_ON_ARRIVAL_S {
            director.fail(track, "stopped as soon as it started");
        } else {
            diag::info!(Audio, "audio: music: {} ended ({played:.0} s)", director.tracks[track].label());
        }
    }

    if director.manifest_dirty && now - director.manifest_written_at >= MANIFEST_EVERY_S {
        director.write_manifest("update");
    }
}

fn start_preparing(director: &mut MusicDirector, frontend: Option<&FrontendAudio>, track: TrackId, context: MusicContext) {
    director.preparing = None;
    let label = director.tracks[track].label();
    diag::info!(Audio, "audio: music: {} prepares {label}", context.label());
    match director.tracks[track].source.clone() {
        Source::File(path) => {
            let pool = AsyncComputeTaskPool::get_or_init(TaskPool::default);
            let task = pool.spawn(async move { read_and_probe(&path) });
            director.preparing = Some(Preparing::File { track, task });
        }
        Source::Game(ns, alias) => {
            let Some(frontend) = frontend else {
                director.fail(track, "no front end sound bank in this run");
                return;
            };
            let Some(key) = clip_keys_for_alias(&frontend.bank, ns, &alias).into_iter().next() else {
                director.fail(track, "not installed (alias not in the front end's sound bank)");
                return;
            };
            let fresh = director
                .game_store
                .as_ref()
                .is_none_or(|(bank, _)| !Arc::ptr_eq(bank, &frontend.bank));
            if fresh {
                director.game_store = Some((
                    Arc::clone(&frontend.bank),
                    ClipStore::start(Arc::clone(&frontend.bank), Some(Arc::clone(&frontend.iwd))),
                ));
            }
            if let Some((_, store)) = director.game_store.as_mut() {
                store.request(key.clone());
            }
            director.preparing = Some(Preparing::Game { track, key });
        }
    }
}

fn poll_preparing(director: &mut MusicDirector, assets: &mut MusicAssets) {
    let Some(preparing) = director.preparing.as_mut() else {
        return;
    };
    match preparing {
        Preparing::File { track, task } => {
            let track = *track;
            let Some(result) = future::block_on(future::poll_once(task)) else {
                return;
            };
            director.preparing = None;
            match result {
                Ok((bytes, p)) => {
                    let ext = match &director.tracks[track].source {
                        Source::File(path) => path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()),
                        Source::Game(..) => None,
                    };
                    let length = p.length;
                    let handle = assets.streams.add(StreamedMusic::new(bytes, ext, &p));
                    if director.core.ready(track, length) {
                        director.prepared.insert(track, Prepared::Stream(handle));
                        let t = &mut director.tracks[track];
                        t.length = length.or(t.length);
                        director.mark(track, true, "decodes");
                    }
                }
                Err(reason) => director.fail(track, &reason),
            }
        }
        Preparing::Game { track, key } => {
            let (track, key) = (*track, key.clone());
            let Some((_, store)) = director.game_store.as_mut() else {
                director.preparing = None;
                director.fail(track, "no clip store");
                return;
            };
            let Some(result) = store.ready(&key) else {
                return;
            };
            store.forget(&key);
            director.preparing = None;
            match result {
                Ok(pcm) => {
                    let frames = pcm.samples().len() / usize::from(pcm.channel_count().max(1));
                    let length = frames as f64 / f64::from(pcm.rate().max(1));
                    let format_note = format!("{} ch, {} Hz", pcm.channel_count(), pcm.rate());
                    let handle = assets.pcm.add(pcm);
                    let t = &mut director.tracks[track];
                    t.length = Some(length);
                    if !t.format.contains("Hz") {
                        t.format = if t.format.is_empty() { format_note } else { format!("{} ({format_note})", t.format) };
                    }
                    director.mark(track, true, "decodes");
                    if director.core.ready(track, Some(length)) {
                        director.prepared.insert(track, Prepared::Pcm(handle));
                    }
                }
                Err(error) => director.fail(track, &format!("did not decode ({error:?})")),
            }
        }
    }
}

/// Map changes and returns need nothing here (the context does it), but the
/// app closing stops every voice at once and writes the final manifest.
pub(crate) fn shutdown_director(
    mut exit: MessageReader<AppExit>,
    director: Option<ResMut<MusicDirector>>,
    mut commands: Commands,
) {
    if exit.read().count() == 0 {
        return;
    }
    let Some(mut director) = director else {
        return;
    };
    for action in director.core.stop_all() {
        if let Action::Stop { voice, .. } = action
            && let Some(live) = director.voices.remove(&voice)
        {
            commands.entity(live.entity).try_despawn();
        }
    }
    director.preparing = None;
    director.prepared.clear();
    director.game_store = None;
    director.write_manifest("shutdown");
    diag::info!(Audio, "audio: music: shut down");
}

pub(crate) fn register(app: &mut App) {
    if app.is_plugin_added::<bevy::audio::AudioPlugin>() {
        bevy::audio::AddAudioSource::add_audio_source::<StreamedMusic>(app);
    } else {
        app.init_asset::<StreamedMusic>();
    }
    app.init_resource::<MenuMusicScreen>().add_systems(
        Update,
        (init_director, update_director, shutdown_director)
            .chain()
            .in_set(ClientSet::Effects),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexts() {
        let ctx = |screen, loading, menu, mm| resolve_context(true, screen, loading, menu, mm);
        use MenuMusicScreen::{Lobby, Main};
        assert_eq!(ctx(Some(AppScreen::MainMenu), false, Main, true), MusicContext::Menu);
        assert_eq!(ctx(Some(AppScreen::MainMenu), false, Lobby, true), MusicContext::Lobby);
        // A load asked for from the lobby: silence at once.
        assert_eq!(ctx(Some(AppScreen::MainMenu), true, Lobby, true), MusicContext::Silent);
        assert_eq!(ctx(Some(AppScreen::Loading), false, Main, true), MusicContext::Silent);
        assert_eq!(ctx(Some(AppScreen::InGame), true, Main, true), MusicContext::Match);
        assert_eq!(ctx(Some(AppScreen::ClassSelect), false, Lobby, true), MusicContext::Match);
        assert_eq!(ctx(Some(AppScreen::InGame), false, Main, false), MusicContext::Silent);
        assert_eq!(ctx(None, false, Main, true), MusicContext::Silent);
        assert_eq!(
            resolve_context(false, Some(AppScreen::MainMenu), false, Main, true),
            MusicContext::Silent
        );
    }
}
