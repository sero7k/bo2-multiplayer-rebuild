//! bo2mp: the front end's Game Folders panel (F10, or Back + Y on a pad).
//!
//! Lists every game the rebuild reads, where it is set to (game_paths.cfg)
//! and whether that folder looks right. Up / Down picks a game; Enter (A)
//! picks its folder with a folder picker; Delete (X) clears it; F10 or
//! Esc (B) closes. The zone search reads the file as it goes, but what was
//! already loaded (the front end, the map list) changes after a restart,
//! which the panel says.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use asset_transport::game_paths;
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::prelude::*;
use frame::ClientSet;

use crate::layers::{GameUiFont, game_text_font};

static OPEN: AtomicBool = AtomicBool::new(false);

/// The panel is up (BO2's menus take no keys meanwhile).
pub(crate) fn is_open() -> bool {
    OPEN.load(Ordering::Relaxed)
}

/// A folder picker running off the main thread: (game key, its answer).
static PICKED: Mutex<Option<(String, Option<PathBuf>)>> = Mutex::new(None);
static PICKING: AtomicBool = AtomicBool::new(false);

#[derive(Component)]
struct PanelRoot;

#[derive(Component)]
struct HintRoot;

#[derive(Default)]
struct PanelState {
    selected: usize,
    /// What the panel shows: (paths as set, a line about the last change).
    drawn: Option<(Vec<(String, PathBuf)>, usize, String, bool)>,
    note: String,
}

#[allow(clippy::too_many_arguments)]
fn game_folders_panel(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<&Gamepad>,
    front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
    font: Option<Res<GameUiFont>>,
    panels: Query<Entity, With<PanelRoot>>,
    hints: Query<Entity, With<HintRoot>>,
    mut state: Local<PanelState>,
) {
    let Some(font) = font else { return };
    let in_front = front.is_some();
    let pressed = |k: KeyCode| keys.just_pressed(k);
    let pad = |b: GamepadButton| pads.iter().any(|p| p.just_pressed(b));
    let pad_held = |b: GamepadButton| pads.iter().any(|p| p.pressed(b));
    if !in_front {
        OPEN.store(false, Ordering::Relaxed);
    } else if pressed(KeyCode::F10) || (pad_held(GamepadButton::Select) && pad(GamepadButton::North)) {
        OPEN.store(!is_open(), Ordering::Relaxed);
        state.note.clear();
    } else if is_open() && (pressed(KeyCode::Escape) || pad(GamepadButton::East)) {
        OPEN.store(false, Ordering::Relaxed);
    }
    // The hint in the front end's corner.
    let hint_wanted = in_front && !is_open();
    if hint_wanted && hints.is_empty() {
        commands
            .spawn((
                HintRoot,
                GlobalZIndex(20_000),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(1.5),
                    top: Val::Percent(1.5),
                    ..default()
                },
            ))
            .with_child((
                Text::new("F10  GAME FOLDERS"),
                game_text_font(&font.0, 15.0),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
            ));
    } else if !hint_wanted {
        for e in &hints {
            commands.entity(e).despawn();
        }
    }
    if !is_open() {
        for e in &panels {
            commands.entity(e).despawn();
        }
        state.drawn = None;
        return;
    }
    let games = game_paths::GAMES;
    if pressed(KeyCode::ArrowUp) || pad(GamepadButton::DPadUp) {
        state.selected = (state.selected + games.len() - 1) % games.len();
    }
    if pressed(KeyCode::ArrowDown) || pad(GamepadButton::DPadDown) {
        state.selected = (state.selected + 1) % games.len();
    }
    let (key, title, _) = games[state.selected];
    // A folder he picked comes back from the picker.
    if let Ok(mut picked) = PICKED.lock()
        && let Some((k, folder)) = picked.take()
    {
        PICKING.store(false, Ordering::Relaxed);
        match folder {
            Some(folder) if game_paths::looks_valid(&folder) => {
                state.note = match game_paths::set(&k, Some(folder.clone())) {
                    Ok(()) => format!("Set to {}. Restart the game to load it.", folder.display()),
                    Err(e) => e,
                };
            }
            Some(folder) => {
                state.note = format!("{} has no `zone` folder: pick the folder the game's .exe is in.", folder.display());
            }
            None => state.note = "No folder picked.".to_owned(),
        }
    }
    let picking = PICKING.load(Ordering::Relaxed);
    if !picking && (pressed(KeyCode::Enter) || pad(GamepadButton::South)) {
        PICKING.store(true, Ordering::Relaxed);
        state.note = format!("Pick {title}'s folder in the window that opened.");
        let (key, title) = (key.to_owned(), title.to_owned());
        std::thread::Builder::new()
            .name("game-folder-picker".into())
            .spawn(move || {
                let folder = game_paths::pick_folder(&format!("Where is {title} installed?"));
                if let Ok(mut p) = PICKED.lock() {
                    *p = Some((key, folder));
                }
            })
            .ok();
    }
    if !picking && (pressed(KeyCode::Delete) || pressed(KeyCode::Backspace) || pad(GamepadButton::West)) {
        state.note = match game_paths::set(key, None) {
            Ok(()) => format!("{title} cleared. Restart the game for it to take effect."),
            Err(e) => e,
        };
    }
    let paths = game_paths::load();
    let drawn = (paths.clone(), state.selected, state.note.clone(), picking);
    if state.drawn.as_ref() == Some(&drawn) && !panels.is_empty() {
        return;
    }
    state.drawn = Some(drawn);
    for e in &panels {
        commands.entity(e).despawn();
    }
    let f = |px: f32| game_text_font(&font.0, px);
    commands
        .spawn((
            PanelRoot,
            GlobalZIndex(20_001),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                BackgroundColor(Color::srgba(0.05, 0.06, 0.07, 0.95)),
                Node {
                    width: Val::Px(900.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((Text::new("GAME FOLDERS"), f(34.0), TextColor(Color::WHITE)));
                p.spawn((
                    Text::new("Where each game is installed. Black Ops II is needed; the others add their maps."),
                    f(16.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                ));
                for (i, (key, title, needed)) in games.iter().enumerate() {
                    let path = paths.iter().find(|(k, _)| k == key).map(|(_, p)| p.clone());
                    let (status, color) = match &path {
                        Some(p) if game_paths::looks_valid(p) => ("OK", Color::srgb(0.35, 0.85, 0.45)),
                        Some(_) => ("NO ZONE FOLDER", Color::srgb(0.95, 0.35, 0.3)),
                        None if *needed => ("NEEDED", Color::srgb(0.95, 0.35, 0.3)),
                        None => ("NOT SET", Color::srgba(1.0, 1.0, 1.0, 0.45)),
                    };
                    let selected = i == state.selected;
                    p.spawn((
                        BackgroundColor(if selected {
                            Color::srgba(0.95, 0.6, 0.1, 0.22)
                        } else {
                            Color::srgba(1.0, 1.0, 1.0, 0.04)
                        }),
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        row.spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|line| {
                            line.spawn((
                                Text::new(title.to_uppercase()),
                                f(20.0),
                                TextColor(if selected { Color::srgb(1.0, 0.75, 0.25) } else { Color::WHITE }),
                            ));
                            line.spawn((Text::new(status), f(16.0), TextColor(color)));
                        });
                        row.spawn((
                            Text::new(path.map_or_else(|| "-".to_owned(), |p| p.display().to_string())),
                            f(15.0),
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        ));
                    });
                }
                if !state.note.is_empty() {
                    p.spawn((Text::new(state.note.clone()), f(16.0), TextColor(Color::srgb(1.0, 0.8, 0.4))));
                }
                p.spawn((
                    Text::new("UP/DOWN choose   ENTER (A) pick folder   DELETE (X) clear   F10 / ESC (B) close"),
                    f(15.0),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                ));
            });
        });
}

pub(crate) fn register_game_folders(app: &mut App) {
    app.add_systems(Update, game_folders_panel.in_set(ClientSet::Ui));
}
