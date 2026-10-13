//! bo2zm: the solo cheat menu, for recording a map's features. Num 1 opens
//! it and pauses the game (as BO2's pause menu stops a solo game, without
//! showing that menu); Num 1 or Esc closes it. Up / Down (arrows, W / S, the
//! wheel) move the highlight, Enter or a left click picks the line and the
//! menu stays open, so a pick can be repeated. The lines are `sim::Cheat`:
//! the game applies them at its next tick. Only in a local game of one
//! player (`sim::cheats_allowed`).

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use frame::ClientSet;
use sim::Cheat;

use crate::layers::{UiLayer, UiLayerVisibility};

/// Whether the menu is up, and its highlighted line. `ate_keys`: it used
/// this frame's keys (the Esc that closed it must not open BO2's pause
/// menu).
#[derive(Resource, Default)]
pub(crate) struct CheatMenu {
    pub open: bool,
    pub ate_keys: bool,
    highlight: usize,
}

#[derive(Component)]
struct CheatRoot;

/// The line `by` lines on from `at`, round the ends.
fn step(at: usize, by: isize, lines: usize) -> usize {
    let n = lines.max(1) as isize;
    (at as isize + by).rem_euclid(n) as usize
}

/// One line as drawn: its label and, for a toggle, ON / OFF.
fn line_text(cheat: Cheat) -> (String, Option<&'static str>) {
    let state = cheat
        .is_toggle()
        .then(|| if sim::cheat_on(cheat) { "ON" } else { "OFF" });
    (cheat.label().to_owned(), state)
}

#[allow(clippy::too_many_arguments)]
fn cheat_menu(
    mut commands: Commands,
    mut menu: ResMut<CheatMenu>,
    (keys, mouse, scroll): (
        Res<ButtonInput<KeyCode>>,
        Res<ButtonInput<MouseButton>>,
        Res<AccumulatedMouseScroll>,
    ),
    (screen, role, view, lui): (
        Res<frame::AppScreen>,
        Option<Res<frame::RuntimeRole>>,
        Option<Res<frame::HudInputView>>,
        Option<Res<frame::LuiMenus>>,
    ),
    mut paused: ResMut<frame::GamePaused>,
    bo2: Option<Res<crate::bo2_font::Bo2Fonts>>,
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<CheatRoot>>,
    mut drawn: Local<Option<String>>,
) {
    menu.ate_keys = false;
    let allowed = *screen == frame::AppScreen::InGame
        && role.as_deref() == Some(&frame::RuntimeRole::Listen)
        && sim::cheats_allowed();
    let console = view.as_deref().is_some_and(|v| v.console_open);
    let lines = Cheat::ALL.len();
    if menu.open && !allowed {
        menu.open = false;
        paused.0 = false;
        diag::info!(Ui, "bo2zm cheats: menu closed (game left)");
    } else if allowed && !console {
        if !menu.open {
            if keys.just_pressed(KeyCode::Numpad1) && !lui.as_deref().is_some_and(|l| l.open) {
                menu.open = true;
                menu.ate_keys = true;
                paused.0 = true;
                diag::info!(Ui, "bo2zm cheats: menu open (game paused)");
            }
        } else {
            menu.ate_keys = true;
            if keys.just_pressed(KeyCode::Numpad1) || keys.just_pressed(KeyCode::Escape) {
                menu.open = false;
                paused.0 = false;
                diag::info!(Ui, "bo2zm cheats: menu closed (game resumed)");
            } else {
                let up = keys.just_pressed(KeyCode::ArrowUp)
                    || keys.just_pressed(KeyCode::KeyW)
                    || scroll.delta.y > 0.0;
                let down = keys.just_pressed(KeyCode::ArrowDown)
                    || keys.just_pressed(KeyCode::KeyS)
                    || scroll.delta.y < 0.0;
                if up {
                    menu.highlight = step(menu.highlight, -1, lines);
                }
                if down {
                    menu.highlight = step(menu.highlight, 1, lines);
                }
                if keys.just_pressed(KeyCode::Enter)
                    || keys.just_pressed(KeyCode::NumpadEnter)
                    || mouse.just_pressed(MouseButton::Left)
                {
                    let cheat = Cheat::ALL[menu.highlight.min(lines - 1)];
                    let on = sim::cheat_pick(cheat);
                    if cheat.is_toggle() {
                        let state = if on { "ON" } else { "OFF" };
                        diag::info!(Ui, "bo2zm cheats: pick {} -> {state}", cheat.label());
                    } else {
                        diag::info!(Ui, "bo2zm cheats: pick {}", cheat.label());
                    }
                }
            }
        }
    }
    draw(
        &mut commands,
        &menu,
        bo2.as_deref(),
        &window,
        &roots,
        &mut drawn,
    );
}

/// The menu at the left of the screen in BO2's menu fonts: the title, then
/// a line each, the highlighted one on a bar.
fn draw(
    commands: &mut Commands,
    menu: &CheatMenu,
    bo2: Option<&crate::bo2_font::Bo2Fonts>,
    window: &Query<&Window, With<PrimaryWindow>>,
    roots: &Query<Entity, With<CheatRoot>>,
    drawn: &mut Option<String>,
) {
    let sh = window.single().map_or(1080.0, |w| w.height());
    let rows: Vec<_> = Cheat::ALL.iter().map(|c| line_text(*c)).collect();
    let sig = format!("{} {} {sh} {rows:?}", menu.open, menu.highlight);
    let Some(bo2) = bo2.filter(|_| menu.open) else {
        if drawn.is_some() {
            for e in roots {
                commands.entity(e).despawn();
            }
            *drawn = None;
        }
        return;
    };
    if drawn.as_deref() == Some(sig.as_str()) {
        return;
    }
    *drawn = Some(sig);
    for e in roots {
        commands.entity(e).despawn();
    }
    let scale = (sh / 1080.0).max(0.4);
    let (white, grey) = (Color::srgb(1.0, 1.0, 1.0), Color::srgb(0.62, 0.62, 0.62));
    let bar = Color::srgba(0.86, 0.5, 0.12, 0.85);
    commands
        .spawn((
            CheatRoot,
            UiLayer::Hud,
            UiLayerVisibility,
            ZIndex(60),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(64.0 * scale),
                top: Val::Px(240.0 * scale),
                width: Val::Px(440.0 * scale),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0 * scale)),
                row_gap: Val::Px(2.0 * scale),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|c| {
            c.spawn(Node {
                margin: UiRect::bottom(Val::Px(10.0 * scale)),
                ..default()
            })
            .with_children(|t| {
                bo2.spawn_line(
                    t,
                    "Big",
                    &[("CHEATS".to_owned(), white)],
                    40.0 * scale,
                    2.0 * scale,
                );
            });
            for (i, (label, state)) in rows.iter().enumerate() {
                let lit = i == menu.highlight;
                let color = if lit { white } else { grey };
                c.spawn((
                    Node {
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::axes(Val::Px(10.0 * scale), Val::Px(3.0 * scale)),
                        ..default()
                    },
                    BackgroundColor(if lit { bar } else { Color::NONE }),
                ))
                .with_children(|row| {
                    bo2.spawn_line(
                        row,
                        "Default",
                        &[(label.clone(), color)],
                        26.0 * scale,
                        1.5 * scale,
                    );
                    if let Some(state) = state {
                        bo2.spawn_line(
                            row,
                            "Default",
                            &[((*state).to_owned(), color)],
                            26.0 * scale,
                            1.5 * scale,
                        );
                    }
                });
            }
        });
}

pub(crate) fn register_cheat_menu_systems(app: &mut App) {
    app.init_resource::<CheatMenu>()
        .add_systems(Update, cheat_menu.in_set(ClientSet::Ui));
}

#[cfg(test)]
mod tests {
    use super::step;

    #[test]
    fn the_highlight_goes_round_the_ends() {
        assert_eq!(step(0, -1, 11), 10);
        assert_eq!(step(10, 1, 11), 0);
        assert_eq!(step(3, 1, 11), 4);
        assert_eq!(step(0, 1, 1), 0);
    }
}
