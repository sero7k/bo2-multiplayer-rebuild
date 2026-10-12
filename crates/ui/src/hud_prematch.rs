//! bo2mp: the match's start countdown in the modern HUD, styled after
//! Advanced Warfare's: "MATCH BEGINS IN" in small white type, the seconds
//! left in big white numerals with a cool blue glow between two thin
//! bracket marks, and a thin line under them that drains across each
//! second. BO2's own yellow countdown is hidden (mp_globallogic.gsc), so
//! only this one shows.
//!
//! The sim publishes `bo2mp_prematch` (milliseconds until play starts,
//! "" when not in the prematch period, from `level.bo2mp_prematchend`);
//! between publishes the HUD counts down on its own clock, so the number
//! and the line stay smooth and reach 0 as play starts.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::GameUiFont;
use crate::shell::theme::ShellFonts;


#[derive(Component)]
struct PrematchRoot;

#[derive(Component)]
struct PrematchLine;

#[derive(Component)]
struct PrematchNumber;

#[derive(Default)]
struct Clock {
    /// The last published value (ms) and the real time it was seen.
    seen: Option<(i64, f64)>,
    shown: i32,
}

/// Milliseconds left now, from the last published value counted down on
/// the real clock.
pub(crate) fn ms_left(published: i64, seen_at: f64, now: f64) -> f64 {
    (published as f64 - (now - seen_at) * 1000.0).max(0.0)
}

/// Whole seconds shown (rounded up; 0 only when play starts).
pub(crate) fn seconds_shown(ms: f64) -> i32 {
    (ms / 1000.0).ceil() as i32
}

/// What is left of the second under way, 1 at its start, 0 at its end
/// (the line under the number drains with it).
pub(crate) fn second_left(ms: f64) -> f32 {
    let r = ms.max(0.0) % 1000.0;
    if ms > 0.0 && r == 0.0 { 1.0 } else { (r / 1000.0) as f32 }
}

const GLOW: Color = Color::srgba(0.45, 0.80, 1.0, 0.55);

#[allow(clippy::too_many_arguments)]
fn prematch_countdown(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    (shell_fonts, game_font): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>),
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<PrematchRoot>>,
    mut lines: Query<&mut Node, With<PrematchLine>>,
    mut numbers: Query<&mut Text, With<PrematchNumber>>,
    time: Res<Time<Real>>,
    mut clock: Local<Clock>,
) {
    let now = time.elapsed_secs_f64();
    let published = match (presented.as_deref().and_then(PresentedSnapshot::snapshot), local.as_deref()) {
        (Some(snap), Some(l)) if crate::hud_modern::modern_hud_enabled() => snap
            .meta
            .script_dvars(l.0)
            .string("bo2mp_prematch")
            .and_then(|v| v.trim().parse::<i64>().ok()),
        _ => None,
    };
    let Some(published) = published else {
        clock.seen = None;
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    if clock.seen.is_none_or(|(v, _)| v != published) {
        clock.seen = Some((published, now));
    }
    let (v, at) = clock.seen.unwrap_or((published, now));
    let ms = ms_left(v, at, now);
    let secs = seconds_shown(ms);
    if secs <= 0 {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    }
    if roots.is_empty() {
        let Some(faces) = crate::hud_match::Faces::of(shell_fonts.as_deref(), game_font.as_deref()) else { return };
        let h = window.single().map_or(1080.0, Window::height);
        let k = (h / 1080.0).max(0.4);
        let bracket = |c: &mut ChildSpawnerCommands, left: bool| {
            // A thin bracket mark: a short upright with ticks at its ends.
            c.spawn(Node {
                width: Val::Px(14.0 * k),
                height: Val::Px(84.0 * k),
                border: if left {
                    UiRect { left: Val::Px(2.0 * k), top: Val::Px(2.0 * k), bottom: Val::Px(2.0 * k), ..default() }
                } else {
                    UiRect { right: Val::Px(2.0 * k), top: Val::Px(2.0 * k), bottom: Val::Px(2.0 * k), ..default() }
                },
                ..default()
            })
            .insert(BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.55)));
        };
        commands
            .spawn((
                PrematchRoot,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    top: Val::Percent(30.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0 * k),
                    ..default()
                },
                GlobalZIndex(30),
                Pickable::IGNORE,
            ))
            .with_children(|c| {
                c.spawn((
                    Text::new("MATCH BEGINS IN"),
                    crate::shell::theme::font(&faces.regular, 22.0 * k),
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                    TextShadow { offset: Vec2::splat(1.5 * k), color: Color::srgba(0.0, 0.0, 0.0, 0.7), ..default() },
                    TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                ));
                c.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(26.0 * k), ..default() }).with_children(|row| {
                    bracket(row, true);
                    row.spawn((
                        PrematchNumber,
                        Text::new(secs.to_string()),
                        crate::shell::theme::font(&faces.bold, 92.0 * k),
                        TextColor(Color::WHITE),
                        TextShadow { offset: Vec2::ZERO, color: GLOW, ..default() },
                        TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                    ));
                    bracket(row, false);
                });
                // The line that drains across each second.
                c.spawn((
                    Node { width: Val::Px(220.0 * k), height: Val::Px(2.0 * k), justify_content: JustifyContent::Center, ..default() },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                ))
                .with_children(|t| {
                    t.spawn((
                        PrematchLine,
                        Node { width: Val::Percent(second_left(ms) * 100.0), height: Val::Percent(100.0), ..default() },
                        BackgroundColor(Color::srgb(0.55, 0.85, 1.0)),
                    ));
                });
            });
        clock.shown = secs;
        return;
    }
    for mut n in &mut lines {
        n.width = Val::Percent(second_left(ms) * 100.0);
    }
    if clock.shown != secs {
        clock.shown = secs;
        for mut t in &mut numbers {
            t.0 = secs.to_string();
        }
    }
}

pub(crate) fn register_prematch_hud(app: &mut App) {
    app.add_systems(Update, prematch_countdown.in_set(frame::ClientSet::Ui));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_between_publishes_and_ends_at_zero() {
        assert_eq!(seconds_shown(ms_left(5000, 10.0, 10.0)), 5);
        assert_eq!(seconds_shown(ms_left(5000, 10.0, 10.001)), 5);
        assert_eq!(seconds_shown(ms_left(5000, 10.0, 11.0)), 4);
        assert_eq!(seconds_shown(ms_left(5000, 10.0, 14.999)), 1);
        assert_eq!(seconds_shown(ms_left(5000, 10.0, 15.0)), 0);
        assert_eq!(seconds_shown(ms_left(5000, 10.0, 20.0)), 0);
    }

    #[test]
    fn the_line_drains_each_second() {
        assert!((second_left(3000.0) - 1.0).abs() < 1e-6);
        assert!((second_left(2500.0) - 0.5).abs() < 1e-6);
        assert!(second_left(0.0).abs() < 1e-6);
    }
}
