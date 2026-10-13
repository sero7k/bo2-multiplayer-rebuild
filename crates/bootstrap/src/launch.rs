use std::path::PathBuf;

use asset_audio::load_mp_sound_bank;
use asset_game::{load_mp_localized_strings, load_ui_menu_catalog};
use asset_transport::{LoadProgress, find_runtime_common_mp, find_zone_file, list_mp_map_packs};
use assets::{
    LoadingPreviewSource, LoadingScreen, MatchLoadRequest, NamespaceSoundIwd, NamespaceTrees,
};
use audio::{SoundBank, SoundIwd};
use bevy::prelude::*;
use bevy::window::PresentMode;
use render::diag::acceptance::{
    ACCEPTANCE_HEIGHT, ACCEPTANCE_PRESENT_MODE, ACCEPTANCE_WIDTH, AcceptanceRun,
};
use render::diag::capture::{CaptureQueue, CaptureRequest};
use render_frontend::prepare::scene::world::WorldScene;
use replay::{Playback, ReplayPlayback};
use session::StartupCommands;
use ui::{
    AppScreen, ClassLoadoutCatalog, LaunchIdentity, LaunchReport, MenuMapList, UiAssetRoot, UiDraw,
    UiLayer, UiLayers,
};

use crate::args::{AcceptanceLaunch, LaunchMode};
use crate::bench;
use crate::plugins::{add_runtime_plugins, add_runtime_plugins_with_role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Listen,
    Dedicated,
    Client,
    Replay,
}

struct LaunchConfig {
    role: Role,
    zone: String,
    games_root: PathBuf,
    artifacts: PathBuf,
}

fn launch_report(
    zone: String,
    common_mp: Result<PathBuf, String>,
    zone_ff: Result<PathBuf, String>,
    zone_alias: Option<String>,
) -> LaunchReport {
    LaunchReport {
        zone,
        common_mp,
        zone_ff,
        zone_alias,
        sim_gap: "map loading has not reached clipmap yet",
        prediction_metrics: None,
        world_report: vec!["queued background zone load".into()],
    }
}

#[derive(Resource)]
struct ShellCommonTask {
    task: bevy::tasks::Task<assets::ShellCommon>,
    perk_table: Option<asset_game::CapturedStringTable>,
    started: std::time::Instant,
}

fn install_class_catalog(
    mut commands: Commands,
    shell: Option<ResMut<ShellCommonTask>>,
    mut strings: ResMut<asset_game::LocalizeCatalog>,
    menus: Res<asset_game::MenuCatalog>,
) {
    use bevy::tasks::futures_lite::future;
    let Some(mut shell) = shell else {
        return;
    };
    let Some(common) = future::block_on(future::poll_once(&mut shell.task)) else {
        return;
    };
    for line in &common.report {
        diag::info!(Launch, "{line}");
    }
    let mut class_catalog =
        ClassLoadoutCatalog::from_weapon_registry(std::sync::Arc::new(common.weapons))
            .with_weapon_tables(&common.tables);
    if let Some(table) = shell.perk_table.as_ref() {
        class_catalog = class_catalog.with_perk_table(table);
    }
    diag::info!(
        Launch,
        "CAC menu: primary={} secondary={} lethal={} tactical={} excluded={} ({:.0}ms after the menu started)",
        class_catalog.primary.len(),
        class_catalog.secondary.len(),
        class_catalog.lethal.len(),
        class_catalog.tactical.len(),
        class_catalog.excluded.len(),
        shell.started.elapsed().as_secs_f32() * 1000.0,
    );
    for (key, preview) in &mut class_catalog.previews {
        if key.contains('+') && menus.material_images.contains_key(&preview.image) {
            preview.image = format!("iw4:material/{}", preview.image);
        }
    }
    strings.absorb(common.strings);
    commands.insert_resource(class_catalog);
    commands.remove_resource::<ShellCommonTask>();
}

fn launch_identity(config: &LaunchConfig) -> LaunchIdentity {
    LaunchIdentity {
        role_label: format!("{:?}", config.role),
        games_root: config.games_root.clone(),
        artifacts: config.artifacts.clone(),
        zone: config.zone.clone(),
    }
}

fn fatal(msg: &str) -> ! {
    diag::exit_launch_error(msg);
}

pub fn launch(
    games: asset_transport::GamesRoot,
    artifacts: PathBuf,
    mode: LaunchMode,
    acceptance: Option<AcceptanceLaunch>,
) {
    diag::info!(Launch, "{}", asset_transport::games_root_report(&games));

    if let Some(plan) = crate::frame_owner::prefer_performance_cores() {
        assets::publish_process_cpus(plan.allowed.clone());
        diag::info!(
            Launch,
            "frame owner: cpus {:?} of {:?} allowed (performance cores {:?})",
            plan.chosen,
            plan.allowed,
            plan.performance
        );
    }
    match mode {
        LaunchMode::Menu => {
            if acceptance.is_some() {
                fatal(&format!(
                    "render acceptance requires `iw4l map <zone>` (not menu); maps: {}",
                    render::diag::acceptance::ACCEPTANCE_MAPS.join(", ")
                ));
            }
            run_menu(games, artifacts);
        }
        LaunchMode::FrontendT6Mp => {
            if acceptance.is_some() {
                fatal("render acceptance requires `iw4l map <zone>` (not frontend)");
            }
            run_frontend_t6mp(games, artifacts);
        }
        LaunchMode::CheckMaps => run_check_maps(&games, &artifacts),
        LaunchMode::CheckGhosts(zone) => run_check_ghosts(&games, &artifacts, zone.as_deref()),
        LaunchMode::CheckGuns => {
            let lines = assets::check_guest_guns();
            for line in &lines {
                println!("{line}");
            }
            let out = artifacts.join("gun-check.txt");
            let _ = std::fs::create_dir_all(&artifacts);
            match std::fs::write(&out, lines.join("\r\n")) {
                Ok(()) => println!("report: {}", out.display()),
                Err(e) => println!("report not written ({}): {e}", out.display()),
            }
        }
        LaunchMode::Map(zone) => {
            // bo2mp: the game is Black Ops II at its core: `iw4l map <zone>`
            // plays the map by Black Ops II's rules, as the front end's
            // START MATCH does, whichever game the map comes from.
            assets::set_t6_rules(true);
            run_map(games, artifacts, zone, acceptance, Role::Listen, None)
        }
        LaunchMode::Serve(zone) => {
            assets::set_t6_rules(true);
            run_map(games, artifacts, zone, None, Role::Dedicated, None)
        }
        LaunchMode::ExportGltf(zone) => {
            if acceptance.is_some() {
                fatal("render acceptance is not available for export-gltf");
            }
            run_export_gltf(games, artifacts, zone);
        }
        LaunchMode::Play {
            name,
            zone_override,
        } => run_play(games, artifacts, name, zone_override, acceptance),
    }
}

/// bo2mp `iw4l check-maps`: every MW2 map the front end lists (MW2's own,
/// IW4x's packs, usermaps) read and checked (`assets::check_iw4_map`), then
/// Call of Duty 4's and World at War's (`assets::check_map`), one
/// line each, to the console and `<artifacts>/map-check.txt`.
fn run_check_maps(games: &asset_transport::GamesRoot, artifacts: &std::path::Path) {
    let mut maps: Vec<String> = asset_transport::list_mp_map_packs(games)
        .into_iter()
        .flat_map(|p| p.maps)
        .filter(|k| k.starts_with("iw4:"))
        .collect();
    maps.sort();
    maps.dedup();
    let mut lines = vec![format!("MW2 map check (with usermaps): {} maps under {}", maps.len(), games.0.display())];
    println!("{}", lines[0]);
    let (mut ok, mut bad) = (0, 0);
    for key in &maps {
        let started = std::time::Instant::now();
        let line = match find_zone_file(games, key) {
            Ok(found) => {
                let path = found.path.clone();
                let result = std::panic::catch_unwind(move || assets::check_iw4_map(&path))
                    .unwrap_or_else(|_| "FAIL the reader panicked".to_owned());
                format!("{key:<28} {result} [{:.1}s] {}", started.elapsed().as_secs_f32(), found.path.display())
            }
            Err(e) => format!("{key:<28} FAIL not found: {e}"),
        };
        if line.contains(" OK:") {
            ok += 1;
        } else {
            bad += 1;
        }
        println!("{line}");
        lines.push(line);
    }
    // bo2mp: Call of Duty 4's and World at War's maps, read by their own
    // readers (`assets::check_map`).
    let legacy = asset_transport::list_legacy_mp_map_files(games);
    let head = format!("CoD4 and World at War map check: {} maps", legacy.len());
    println!("{head}");
    lines.push(head);
    for (game, stem, path) in legacy {
        let started = std::time::Instant::now();
        let key = format!("{game}:{stem}");
        let at = path.clone();
        let result = std::panic::catch_unwind(move || assets::check_map(&game, &at))
            .unwrap_or_else(|_| "FAIL the reader panicked".to_owned());
        let line = format!("{key:<28} {result} [{:.1}s] {}", started.elapsed().as_secs_f32(), path.display());
        if line.contains(" OK:") {
            ok += 1;
        } else {
            bad += 1;
        }
        println!("{line}");
        lines.push(line);
    }
    lines.push(format!("{ok} OK, {bad} with problems"));
    println!("{}", lines.last().map_or("", String::as_str));
    let out = artifacts.join("map-check.txt");
    let _ = std::fs::create_dir_all(artifacts);
    match std::fs::write(&out, lines.join("\r\n")) {
        Ok(()) => println!("report: {}", out.display()),
        Err(e) => println!("report not written ({}): {e}", out.display()),
    }
}

/// `iw4l check-ghosts [zone]`: see `audio::music::ghosts_check_report`;
/// to the console and `<artifacts>/ghosts-check.txt`.
fn run_check_ghosts(games: &asset_transport::GamesRoot, artifacts: &std::path::Path, zone: Option<&str>) {
    let lines = audio::music::ghosts_check_report(Some(&games.0), zone);
    for line in &lines {
        println!("{line}");
    }
    let out = artifacts.join("ghosts-check.txt");
    let _ = std::fs::create_dir_all(artifacts);
    match std::fs::write(&out, lines.join("\r\n")) {
        Ok(()) => println!("report: {}", out.display()),
        Err(e) => println!("report not written ({}): {e}", out.display()),
    }
}

fn run_export_gltf(games: asset_transport::GamesRoot, artifacts: PathBuf, zone_arg: String) {
    let found = find_zone_file(&games, &zone_arg)
        .unwrap_or_else(|error| fatal(&format!("export-gltf: {error}")));
    let game = asset_transport::zone_game_for_path(&found.path)
        .unwrap_or_else(|| fatal("export-gltf: source game could not be identified"));
    if game != asset_core::ZoneGame::Iw4 {
        fatal(&format!(
            "export-gltf P1a supports native IW4 only; {} is {}",
            found.zone_name,
            game.prefix()
        ));
    }
    let common_mp = find_runtime_common_mp(&games, &found.path).map(|zone| zone.path);

    let prepared = match bevy::tasks::futures_lite::future::block_on(assets::load_prepared_match(
        Ok(found.path),
        common_mp,
        asset_transport::LoadProgress::default(),
    )) {
        assets::MatchLoadOutcome::Ready(prepared) => prepared,
        assets::MatchLoadOutcome::Canceled => fatal("export-gltf: map walk canceled"),
    };
    let summary = assets::export_prepared_world_gltf(
        &artifacts,
        &found.zone_name,
        prepared.world,
        &prepared.materials,
    )
    .unwrap_or_else(|error| fatal(&format!("export-gltf: {error}")));
    diag::announce_stdout(&summary.scene.display().to_string());
    diag::announce_stdout(&summary.report_line());
}

fn run_menu(games: asset_transport::GamesRoot, artifacts: PathBuf) {
    start_perf(None, "menu");
    let ui_games = asset_game::ui_games_root(&games).unwrap_or_else(|error| {
        let content = asset_transport::games_content_report(&games).join("\n");
        fatal(&format!(
            "Cannot start the IW4 menu: base MW2 Multiplayer assets were not found.\n\n\
             {content}\n\n\
             The menu requires common_mp.ff with IW4 envelope version 0x114. \
             Point IW4L_GAMES or a shortcut beside iw4launcher.exe to the folder containing \
             the base MW2 Multiplayer files. If this is the intended folder, restore its \
             missing base files; DLC maps alone are insufficient.\n\nSearch details: {error}"
        ))
    });
    let shell_common = assets::load_pool().spawn(assets::load_shell_common(games.clone()));
    let (mut menus, menu_report) = load_ui_menu_catalog(&ui_games);
    ui::install_frontend_menus(&mut menus).unwrap_or_else(|error| fatal(&error));
    for line in &menu_report {
        diag::info!(Launch, "{line}");
    }
    let missing = ["main", "main_text"]
        .into_iter()
        .filter(|name| menus.get(name).is_none())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        fatal(&format!(
            "Cannot start the IW4 menu: {} not loaded from {}.\n\n\
             A missing menu can mean an absent file or a FastFile decoding failure. \
             Include the catalog details below when reporting this error.\n\n{}",
            missing.join(", "),
            ui_games.0.display(),
            menu_report.join("\n")
        ));
    }
    let maps = list_mp_map_packs(&games);
    diag::info!(
        Launch,
        "menu: {} maps in {} packs under {}",
        maps.iter().map(|pack| pack.maps.len()).sum::<usize>(),
        maps.len(),
        games.0.display()
    );
    let config = LaunchConfig {
        role: Role::Listen,
        zone: String::new(),
        games_root: games.0.clone(),
        artifacts,
    };
    let mut app = App::new();
    let namespace_trees = NamespaceTrees::discover(&games);
    let have = content_flags(&namespace_trees, &games);
    let master_intent = if have.0 & net::CONTENT_IW4 == 0 {
        diag::warn!(
            Net,
            "master browser disabled: IW4 common_mp.ff is not installed"
        );
        net::MasterLaunchIntent::disabled()
    } else {
        net::MasterLaunchIntent::browser_from_env(have).unwrap_or_else(|error| {
            diag::warn!(Net, "master browser disabled: {error}");
            net::MasterLaunchIntent::disabled()
        })
    };
    app.insert_resource(master_intent);
    app.add_plugins(crate::plugins::default_plugins_with_quiet_log(
        WindowPlugin {
            primary_window: Some(Window {
                title: "iw4l".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        },
    ));
    app.insert_resource(ShellCommonTask {
        task: shell_common,
        perk_table: menus.string_table("mp/perkTable.csv").cloned(),
        started: std::time::Instant::now(),
    })
    .add_systems(Update, install_class_catalog);
    match load_mp_localized_strings(&ui_games, "iw4:code_post_gfx_mp") {
        Ok(loc) => {
            diag::info!(Launch, "menu: {} localize keys", loc.len());
            app.insert_resource(loc);
        }
        Err(error) => diag::warn!(Launch, "menu: localize: {error}"),
    }
    app.insert_resource(menus);
    let menu_bank = match load_mp_sound_bank(&ui_games, "iw4:code_post_gfx_mp") {
        Ok(loaded) => {
            diag::info!(Launch, "menu: sound bank ready");
            for line in loaded.gap_lines() {
                diag::warn!(Launch, "menu: {line}");
            }
            let bank = std::sync::Arc::new(loaded.catalog);
            app.insert_resource(SoundBank(std::sync::Arc::clone(&bank)));
            Some(bank)
        }
        Err(error) => {
            diag::warn!(Launch, "menu: sound bank: {error}");
            None
        }
    };

    let (menu_iwd, lines) = NamespaceSoundIwd::open(&namespace_trees);
    for line in lines {
        diag::info!(Launch, "menu: {line}");
    }
    let menu_iwd = std::sync::Arc::new(menu_iwd);
    app.insert_resource(SoundIwd(std::sync::Arc::clone(&menu_iwd)));
    if let Some(bank) = menu_bank {
        app.insert_resource(audio::ClipStore::start(
            std::sync::Arc::clone(&bank),
            Some(std::sync::Arc::clone(&menu_iwd)),
        ));
        app.insert_resource(audio::FrontendAudio {
            bank,
            iwd: menu_iwd,
        });
    }
    app.insert_resource(launch_identity(&config))
        .insert_resource(MenuMapList(maps))
        .insert_resource(AppScreen::MainMenu)
        .insert_resource(UiAssetRoot(Some(ui_games.0)))
        .insert_resource(StartupCommands {
            lines: console::startup_commands(),
        })
        .insert_resource(WorldScene::default())
        .insert_resource(ClearColor(Color::srgb(0.04, 0.045, 0.06)))
        .insert_resource({
            let mut layers = UiLayers::default();
            layers.show_only([UiLayer::Shell, UiLayer::Overlay]);
            layers
        });
    add_runtime_plugins(&mut app);
    if let Some(capture) = CaptureRequest::from_env() {
        queue_launch_capture(
            &mut app,
            CaptureRequest {
                exit_after_capture: true,
                ..capture
            },
        );
    }
    app.run();
    let _ = flush_perf();
}

/// bo2mp lane C: Black Ops II's multiplayer front end and no map: his MP UI
/// zones (scripts, pictures, fonts, tables, English text) read here, then
/// the menu screen runs BO2's own menus (`ui::Bo2mpFrontend`, drawn by the
/// LUI HUD on the shell layer). No MW2 menu catalog, sounds or master
/// browser. START MATCH swaps to `t6:<map>` like the `map` command.
fn run_frontend_t6mp(games: asset_transport::GamesRoot, artifacts: PathBuf) {
    start_perf(None, "frontend");
    // Every match this front end starts plays by Black Ops II's rules,
    // whichever game its map comes from.
    assets::set_t6_rules(true);
    // The first launch asks where each game is installed.
    crate::game_folders::first_run();
    let patch = find_zone_file(&games, "t6:patch_mp").unwrap_or_else(|error| {
        fatal(&format!(
            "Cannot start the Black Ops II multiplayer front end: its files were not found.\n\nSet Black Ops II's folder in game_paths.cfg beside the game, or delete that file to be asked again.\n\n{error}"
        ))
    });
    let zone_all = patch.path.parent().map(std::path::Path::to_path_buf).unwrap_or_default();
    let started = std::time::Instant::now();
    let front = assets::load_t6_frontend(&zone_all);
    for line in &front.report {
        diag::info!(Launch, "{line}");
    }
    diag::info!(
        Launch,
        "frontend t6mp: read {} in {:.1} s",
        zone_all.display(),
        started.elapsed().as_secs_f64()
    );
    let config = LaunchConfig {
        role: Role::Listen,
        zone: String::new(),
        games_root: games.0.clone(),
        artifacts: artifacts.clone(),
    };
    let mut app = App::new();
    // The front end's online panel counts the players on the master server
    // (IW4L_MASTER_ADDR), when one is set.
    let have = content_flags(&NamespaceTrees::discover(&games), &games);
    app.insert_resource(net::MasterLaunchIntent::browser_from_env(have).unwrap_or_else(|error| {
        diag::warn!(Net, "front end online panel: no master server ({error})");
        net::MasterLaunchIntent::disabled()
    }));
    app.add_plugins(crate::plugins::default_plugins_with_quiet_log(
        WindowPlugin {
            primary_window: Some(Window {
                title: "iw4l — Black Ops II".into(),
                resolution: (1280, 720).into(),
                position: test_window_position().unwrap_or_default(),
                focused: test_window_position().is_none() && !test_window_hidden(),
                visible: !test_window_hidden(),
                ..default()
            }),
            ..default()
        },
    ));
    let fe = ui::Bo2mpFrontend {
        stats_path: artifacts.join("bo2mp_stats.txt"),
    };
    // BO2's menu sounds and music: the front end's own bank (also put back
    // when a match ends).
    match front.sound {
        Ok(mut catalog) => {
            // The menu music rotates through BO1's, MW2's and MW3's main-menu
            // tracks too: their catalogs join BO2's, and their games' sound
            // archives (BO1 from the games folder, MW2/MW3 from the
            // voice-only one) are opened for the streamed tracks.
            let (guests, lines) = asset_audio::menu_music_guests(&games);
            for line in lines {
                diag::info!(Launch, "{line}");
            }
            for guest in guests {
                catalog.absorb_missing_aliases_unresolved(guest);
            }
            catalog.finalize();
            let mut trees = asset_transport::NamespaceTrees::discover(&games);
            if let Some(voice) = asset_audio::voice_games_root() {
                let voice = asset_transport::NamespaceTrees::discover(&voice);
                for ns in [asset_core::AssetNamespace::Iw4, asset_core::AssetNamespace::Iw5] {
                    if trees.get(ns).is_none()
                        && let Some(tree) = voice.get(ns)
                    {
                        trees.adopt_zone(&tree.anchor);
                    }
                }
            }
            let (iwd, lines) = asset_transport::NamespaceSoundIwd::open(&trees);
            for line in lines {
                diag::info!(Launch, "{line}");
            }
            let bank = std::sync::Arc::new(catalog);
            let iwd = std::sync::Arc::new(iwd);
            app.insert_resource(SoundBank(std::sync::Arc::clone(&bank)));
            app.insert_resource(SoundIwd(std::sync::Arc::clone(&iwd)));
            app.insert_resource(audio::ClipStore::start(std::sync::Arc::clone(&bank), Some(std::sync::Arc::clone(&iwd))));
            app.insert_resource(audio::FrontendAudio { bank, iwd });
        }
        Err(error) => diag::warn!(Launch, "frontend t6mp: no menu sounds: {error}"),
    }
    app.insert_resource(ui::Bo2mpFrontendAssets {
        ui: front.ui.clone(),
        icons: front.icons.clone(),
        fonts: front.fonts.clone(),
        front: fe.clone(),
    })
    .insert_resource(front.ui)
        .insert_resource(front.icons)
        .insert_resource(front.fonts)
        .insert_resource(fe)
        .insert_resource(asset_game::MenuCatalog::default())
        .insert_resource(launch_identity(&config))
        .insert_resource(MenuMapList(Vec::new()))
        .insert_resource(AppScreen::MainMenu)
        .insert_resource(UiAssetRoot(None))
        .insert_resource(StartupCommands {
            lines: console::startup_commands(),
        })
        .insert_resource(WorldScene::default())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource({
            let mut layers = UiLayers::default();
            layers.show_only([UiLayer::Shell, UiLayer::Overlay]);
            layers
        });
    add_runtime_plugins(&mut app);
    if let Some(capture) = CaptureRequest::from_env() {
        queue_launch_capture(
            &mut app,
            CaptureRequest {
                exit_after_capture: true,
                ..capture
            },
        );
    }
    app.run();
    let _ = flush_perf();
}

fn run_play(
    games: asset_transport::GamesRoot,
    artifacts: PathBuf,
    name: String,
    zone_override: Option<String>,
    acceptance: Option<AcceptanceLaunch>,
) {
    if acceptance.is_some() {
        fatal("render acceptance requires `iw4l map <zone>` (not play)");
    }
    let playback = Playback::open(&artifacts, &name).unwrap_or_else(|e| {
        fatal(&format!("play: {e}"));
    });
    let zone = zone_override
        .or_else(|| playback.identity().zone_name().map(str::to_owned))
        .unwrap_or_else(|| {
            fatal(&format!(
                "play: recording {} has no zone — re-record on this build, or: make play {name} ZONE=<map>",
                playback.path().display()
            ));
        });
    diag::info!(Launch, "play: {} zone={zone}", playback.path().display());
    let session = ReplayPlayback::new(playback);
    run_map(games, artifacts, zone, None, Role::Replay, Some(session));
}

fn run_map(
    games: asset_transport::GamesRoot,
    artifacts: PathBuf,
    zone_arg: String,
    acceptance: Option<AcceptanceLaunch>,
    role: Role,
    playback: Option<ReplayPlayback>,
) {
    let found = find_zone_file(&games, &zone_arg);
    let zone_alias = found.as_ref().ok().and_then(|z| z.alias_note.clone());
    let zone_arg_lc = zone_arg.trim().to_ascii_lowercase();
    let (parsed_game, requested_stem) = asset_transport::split_zone_key(&zone_arg_lc);
    let zone = found
        .as_ref()
        .ok()
        .map(|z| z.zone_name.clone())
        .unwrap_or_else(|| requested_stem.to_owned());
    let zone_ff = found
        .as_ref()
        .map(|z| z.path.clone())
        .map_err(|e| e.clone());
    let common_mp = match &zone_ff {
        Ok(path) => find_runtime_common_mp(&games, path).map(|z| z.path),
        Err(e) => Err(e.clone()),
    };
    let probe = launch_report(zone.clone(), common_mp.clone(), zone_ff.clone(), zone_alias);
    let game = parsed_game.or_else(|| {
        found
            .as_ref()
            .ok()
            .and_then(|z| asset_transport::zone_game_for_path(&z.path))
    });
    let loading_title = asset_transport::map_load_title(&zone_arg_lc, game);
    let namespace_trees = NamespaceTrees::discover(&games);
    let have = content_flags(&namespace_trees, &games);
    let requires = net::content_required_by_map(&zone_arg_lc)
        .unwrap_or_else(|error| fatal(&format!("master content: {error}")));

    let master_intent = if role == Role::Replay {
        net::MasterLaunchIntent::disabled()
    } else if have.0 & net::CONTENT_IW4 == 0 {
        diag::warn!(
            Net,
            "master launch disabled: IW4 common_mp.ff is not installed"
        );
        net::MasterLaunchIntent::disabled()
    } else {
        net::MasterLaunchIntent::from_env_for_map(&zone, have, requires).unwrap_or_else(|error| {
            diag::warn!(Net, "master launch disabled: {error}");
            net::MasterLaunchIntent::disabled()
        })
    };

    let config = LaunchConfig {
        role: if master_intent.is_join() {
            Role::Client
        } else {
            role
        },
        zone: zone.clone(),
        games_root: games.0.clone(),
        artifacts,
    };
    start_perf(Some(zone.clone()), role_name(config.role));
    let (mut menus, menu_report) = load_ui_menu_catalog(&games);
    if let Err(error) = ui::install_frontend_menus(&mut menus) {
        // bo2zm: a Black Ops II map runs without MW2's menu files, which the
        // built-in front-end menus extend.
        if game == Some(asset_transport::ZoneGame::T6) {
            diag::warn!(Launch, "front-end menus left out for a T6 map: {error}");
        } else {
            fatal(&error);
        }
    }
    for line in &menu_report {
        diag::info!(Launch, "{line}");
    }
    let progress = LoadProgress::default();
    let acceptance_run = acceptance
        .map(|a| AcceptanceRun {
            artifact_dir: a.dir,
            map: zone.clone(),
        })
        .or_else(|| AcceptanceRun::from_env(zone.clone()));
    let present_mode = launch_present_mode(acceptance_run.is_some());
    let mut app = App::new();
    if acceptance_run.is_some() || std::env::var_os("IW4L_PRESENT_MODE").is_some() {
        app.insert_resource(ui::PresentModeOverride(present_mode));
    }
    app.insert_resource(master_intent);
    let dedicated = config.role == Role::Dedicated;
    if dedicated {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .insert_resource(frame::Headless);
    }
    if frame::Headless::requested() {
        // IW4L_HEADLESS=1: nothing waits for the world to be drawn or heard.
        diag::info!(Launch, "headless: no window, no graphics device, no sound device (IW4L_HEADLESS=1)");
        app.insert_resource(frame::Headless);
    }
    app.add_plugins(crate::plugins::default_plugins_with_quiet_log(
        WindowPlugin {
            exit_condition: if dedicated {
                bevy::window::ExitCondition::DontExit
            } else {
                bevy::window::ExitCondition::OnAllClosed
            },
            primary_window: (!dedicated).then(|| Window {
                title: match config.role {
                    Role::Replay => format!("iw4l — play {}", config.zone),
                    Role::Listen | Role::Dedicated => format!("iw4l — {}", config.zone),
                    Role::Client => format!("iw4l — join {}", config.zone),
                },
                resolution: (ACCEPTANCE_WIDTH, ACCEPTANCE_HEIGHT).into(),
                present_mode,
                // IW4L_WINDOW_POS=x,y places the window (test runs on another
                // monitor) and opens it without taking the keyboard.
                position: test_window_position().unwrap_or_default(),
                focused: test_window_position().is_none() && !test_window_hidden(),
                // IW4L_WINDOW_HIDDEN=1: a test run with no window to see
                // (log-only tests while he plays something else).
                visible: !test_window_hidden(),
                ..default()
            }),
            ..default()
        },
    ));
    if let Ok(path) = &zone_ff {
        app.insert_resource(LoadingPreviewSource {
            path: path.clone(),
            map_name: zone.clone(),
            request_id: 0,
        });
    }
    let ui_games_root = asset_game::ui_games_root(&games).ok().map(|root| root.0);
    app.insert_resource(launch_identity(&config))
        .insert_resource(probe)
        .insert_resource(menus)
        .insert_resource(MatchLoadRequest {
            request_id: 0,
            load_key: Default::default(),
            zone: zone.clone(),
            zone_ff,
            common_mp,
            progress: progress.clone(),
        })
        .insert_resource(LoadingScreen::new(
            progress.clone(),
            loading_title,
            sim::host_game_mode_kind().display_name().to_owned(),
        ))
        .insert_resource(UiAssetRoot(ui_games_root))
        .insert_resource(StartupCommands {
            lines: console::startup_commands(),
        })
        .insert_resource(WorldScene::default())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AppScreen::Loading)
        .insert_resource({
            let mut layers = UiLayers::default();
            layers.show_only([UiLayer::Loading, UiLayer::Overlay]);
            layers
        });
    if let Some(run) = acceptance_run {
        diag::info!(
            Launch,
            "render acceptance: dir={} map={} {}x{} {:?}",
            run.artifact_dir.display(),
            run.map,
            ACCEPTANCE_WIDTH,
            ACCEPTANCE_HEIGHT,
            present_mode
        );

        app.insert_resource(UiDraw(false));
        app.insert_resource(run);
    }
    match config.role {
        Role::Replay => add_runtime_plugins_with_role(&mut app, frame::RuntimeRole::Replay),
        Role::Listen | Role::Dedicated => add_runtime_plugins(&mut app),
        Role::Client => add_runtime_plugins_with_role(&mut app, frame::RuntimeRole::Client),
    }
    // The demo, before the playback moves into the world: it names the workload
    // in the bench manifest, and "same demo" is what makes two runs comparable
    // at all. The whole path, not a stem — a clip is `clips/<id>/clip.iw4ldemo`,
    // whose stem is `clip` for every clip ever recorded.
    let demo = playback
        .as_ref()
        .map(|playback| playback.path().display().to_string());
    if let Some(playback) = playback {
        app.insert_resource(playback);
    }
    if let Some(capture) = CaptureRequest::from_env() {
        queue_launch_capture(&mut app, capture);
    }
    bench::announce_runtime(&mut app);
    let bench = bench::enabled();
    if bench {
        bench::insert(
            &mut app,
            &zone,
            demo.as_deref(),
            role_name(config.role),
            &config.artifacts,
            progress.clone(),
        );
    }
    app.run();
    let trace = flush_perf();
    if bench {
        bench::finish(&config.artifacts, trace);
    }
}

fn content_flags(trees: &NamespaceTrees, games: &asset_transport::GamesRoot) -> master_protocol::ContentFlags {
    let mut flags = net::content_inventory(
        trees.get(asset_core::AssetNamespace::Iw4).is_some(),
        trees.get(asset_core::AssetNamespace::Iw5).is_some(),
        trees.get(asset_core::AssetNamespace::T5).is_some(),
    );
    // bo2zm: Black Ops II.
    if trees.get(asset_core::AssetNamespace::T6).is_some() {
        flags.0 |= net::CONTENT_T6;
    }
    // bo2mp: Call of Duty 4 and World at War, when any of their maps is in.
    for (game, _) in asset_transport::list_legacy_mp_maps(games) {
        flags.0 |= match game.as_str() {
            "iw3" => net::CONTENT_IW3,
            "t4" => net::CONTENT_T4,
            _ => 0,
        };
    }
    flags
}

fn start_perf(zone: Option<String>, role: &str) {
    if let Err(error) = perf::start(perf::RunMetadata {
        zone,
        role: role.to_owned(),
        focus: std::env::var("IW4L_PERF_FOCUS").ok(),
    }) {
        diag::error!(Launch, "perf: start failed: {error}");
    }
}

fn flush_perf() -> Option<PathBuf> {
    match perf::flush() {
        Ok(path) => path,
        Err(error) => {
            diag::error!(Launch, "perf: flush failed: {error}");
            None
        }
    }
}

fn queue_launch_capture(app: &mut App, request: CaptureRequest) {
    app.world_mut()
        .get_resource_mut::<CaptureQueue>()
        .expect("CaptureQueue: RenderPlugin must be added before a launch capture is queued")
        .push(request);
}

const fn role_name(role: Role) -> &'static str {
    match role {
        Role::Listen => "listen",
        Role::Dedicated => "dedicated",
        Role::Client => "client",
        Role::Replay => "replay",
    }
}

fn launch_present_mode(acceptance: bool) -> PresentMode {
    if acceptance {
        return ACCEPTANCE_PRESENT_MODE;
    }
    match std::env::var("IW4L_PRESENT_MODE").ok().as_deref() {
        Some("AutoNoVsync") => PresentMode::AutoNoVsync,
        Some("Immediate") => PresentMode::Immediate,
        Some("Mailbox") => PresentMode::Mailbox,
        Some("Fifo") => PresentMode::Fifo,
        Some(other) => {
            diag::warn!(Launch, "unknown IW4L_PRESENT_MODE={other}; using Fifo");
            PresentMode::default()
        }
        None => PresentMode::default(),
    }
}

/// `IW4L_WINDOW_POS=x,y`: where a test run's window opens.
fn test_window_position() -> Option<bevy::window::WindowPosition> {
    let v = std::env::var("IW4L_WINDOW_POS").ok()?;
    let (x, y) = v.split_once(',')?;
    Some(bevy::window::WindowPosition::At(bevy::math::IVec2::new(
        x.trim().parse().ok()?,
        y.trim().parse().ok()?,
    )))
}

/// `IW4L_WINDOW_HIDDEN=1`: a test run's window stays hidden.
pub(crate) fn test_window_hidden() -> bool {
    std::env::var("IW4L_WINDOW_HIDDEN").is_ok_and(|v| v == "1")
}
