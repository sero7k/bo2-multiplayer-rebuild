//! bo2mp: a line along the bottom of the loading screen saying what the load
//! is doing and what has taken longest, so a slow map load shows why: the
//! total time so far, the stages running now (with their counts), and the
//! three slowest finished stages. When the load ends its slowest stages go
//! to the log as one line.

use bevy::prelude::*;
use frame::ClientSet;

use crate::layers::{GameUiFont, game_text_font};

#[derive(Component)]
struct InfoBar;

#[derive(Component)]
struct InfoBarRoot;

fn seconds(d: std::time::Duration) -> String {
    format!("{:.1}s", d.as_secs_f32())
}

fn load_info_bar(
    mut commands: Commands,
    loading: Option<Res<assets::LoadingScreen>>,
    font: Option<Res<GameUiFont>>,
    roots: Query<Entity, With<InfoBarRoot>>,
    mut bars: Query<&mut Text, With<InfoBar>>,
    mut last: Local<Option<(u64, String)>>,
) {
    let Some(loading) = loading else {
        // The load is over: its slowest stages once, to the log.
        if let Some((id, line)) = last.take() {
            diag::info!(Ui, "load #{id} took: {line}");
        }
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    let Some(font) = font else { return };
    let snap = loading.progress.snapshot();
    let total = snap.now.saturating_duration_since(snap.requested_at);
    let running: Vec<String> = snap
        .stages
        .iter()
        .filter(|s| s.running())
        .map(|s| {
            let count = s.count.map_or_else(String::new, |c| match c.total {
                Some(t) => format!(" {}/{}", c.completed, t),
                None => format!(" {}", c.completed),
            });
            format!("{}{count} ({})", s.key.label(), s.elapsed(snap.now).map_or_else(String::new, seconds))
        })
        .take(3)
        .collect();
    let mut done: Vec<(String, std::time::Duration)> = snap
        .stages
        .iter()
        .filter(|s| s.end.is_some())
        .filter_map(|s| Some((s.key.label(), s.elapsed(snap.now)?)))
        .collect();
    done.sort_by(|a, b| b.1.cmp(&a.1));
    let slowest: Vec<String> = done.iter().take(3).map(|(l, d)| format!("{l} {}", seconds(*d))).collect();
    let line = format!(
        "LOADING {}   |   NOW: {}   |   SLOWEST: {}",
        seconds(total),
        if running.is_empty() { "waiting for the world to appear".to_owned() } else { running.join(", ") },
        if slowest.is_empty() { "-".to_owned() } else { slowest.join(", ") }
    );
    *last = Some((snap.request_id, format!("{} in all; slowest {}", seconds(total), slowest.join(", "))));
    if !roots.is_empty() {
        let Some(mut text) = bars.iter_mut().next() else { return };
        if text.0 != line {
            text.0 = line;
        }
        return;
    }
    commands
        .spawn((
            InfoBarRoot,
            GlobalZIndex(20_002),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn((
                InfoBar,
                Text::new(line),
                game_text_font(&font.0, 15.0),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
        });
}

pub(crate) fn register_load_info(app: &mut App) {
    app.add_systems(Update, load_info_bar.in_set(ClientSet::Ui));
}
