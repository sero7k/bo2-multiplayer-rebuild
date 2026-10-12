//! bo2mp: the match's start countdown in the modern HUD: "MATCH BEGINS IN"
//! over a dark disc with Black Ops II's spinning ring (a bright arc that
//! turns once a second around a dim track) and the whole seconds left in
//! the middle, in blue on black like the rest of our HUD.
//!
//! The sim publishes `bo2mp_prematch` (milliseconds until play starts,
//! "" when not in the prematch period, from `level.bo2mp_prematchend`);
//! between publishes the HUD counts down on its own clock, so the number
//! and the ring stay smooth and reach 0 as play starts.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::layers::GameUiFont;
use crate::shell::theme::ShellFonts;

const BLUE: Color = Color::srgb(0.30, 0.64, 1.0);

#[derive(Component)]
struct PrematchRoot;

#[derive(Component)]
struct PrematchRing;

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

/// The spinning arc's start angle (radians): one full turn a second.
pub(crate) fn spin_angle(now: f64) -> f32 {
    (now.fract() as f32) * std::f32::consts::TAU
}

fn ring(now: f64) -> BackgroundGradient {
    let a = spin_angle(now);
    let arc = 1.4_f32;
    let track = Color::srgba(0.30, 0.64, 1.0, 0.16);
    let tau = std::f32::consts::TAU;
    let (s, e) = (a, a + arc);
    let stops = if e <= tau {
        vec![
            AngularColorStop::new(track, 0.0),
            AngularColorStop::new(track, s),
            AngularColorStop::new(BLUE.with_alpha(0.25), s),
            AngularColorStop::new(BLUE, e),
            AngularColorStop::new(track, e),
            AngularColorStop::new(track, tau),
        ]
    } else {
        let wrap = e - tau;
        vec![
            AngularColorStop::new(BLUE.with_alpha(0.25 + 0.75 * (1.0 - wrap / arc)), 0.0),
            AngularColorStop::new(BLUE, wrap),
            AngularColorStop::new(track, wrap),
            AngularColorStop::new(track, s),
            AngularColorStop::new(BLUE.with_alpha(0.25), s),
            AngularColorStop::new(BLUE.with_alpha(0.25 + 0.75 * ((tau - s) / arc)), tau),
        ]
    };
    BackgroundGradient::from(ConicGradient::new(UiPosition::CENTER, stops))
}

#[allow(clippy::too_many_arguments)]
fn prematch_countdown(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    (shell_fonts, game_font): (Option<Res<ShellFonts>>, Option<Res<GameUiFont>>),
    window: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<PrematchRoot>>,
    mut rings: Query<&mut BackgroundGradient, With<PrematchRing>>,
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
        let size = 150.0 * k;
        commands
            .spawn((
                PrematchRoot,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    top: Val::Percent(24.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0 * k),
                    ..default()
                },
                GlobalZIndex(30),
                Pickable::IGNORE,
            ))
            .with_children(|c| {
                c.spawn((
                    Text::new("MATCH BEGINS IN"),
                    crate::shell::theme::font(&faces.bold, 26.0 * k),
                    TextColor(Color::WHITE),
                    TextShadow { offset: Vec2::splat(2.0 * k), color: Color::srgba(0.0, 0.0, 0.0, 0.8), ..default() },
                    TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                ));
                // The ring: the spinning arc around a dark disc.
                c.spawn((
                    PrematchRing,
                    Node {
                        width: Val::Px(size),
                        height: Val::Px(size),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ring(now),
                ))
                .with_children(|r| {
                    r.spawn((
                        Node {
                            width: Val::Px(size - 12.0 * k),
                            height: Val::Px(size - 12.0 * k),
                            border_radius: BorderRadius::all(Val::Percent(50.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.01, 0.03, 0.88)),
                    ))
                    .with_children(|d| {
                        d.spawn((
                            PrematchNumber,
                            Text::new(secs.to_string()),
                            crate::shell::theme::font(&faces.bold, 76.0 * k),
                            TextColor(BLUE),
                            TextLayout::linebreak(bevy::text::LineBreak::NoWrap),
                        ));
                    });
                });
            });
        clock.shown = secs;
        return;
    }
    for mut g in &mut rings {
        *g = ring(now);
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
    fn the_arc_turns_once_a_second() {
        assert!(spin_angle(3.0).abs() < 1e-5);
        assert!((spin_angle(3.5) - std::f32::consts::PI).abs() < 1e-3);
    }
}
