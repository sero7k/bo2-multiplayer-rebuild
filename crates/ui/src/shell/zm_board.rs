//! bo2mp shell: Zombies' scoreboard and game over, in the shell's look,
//! laid out as Black Ops' Zombies scoreboard (no map emblem): the map and
//! mode over a table of the players - their colour, name, points, kills,
//! downs, revives and headshots - while the scores key (Tab) is held, and
//! at the game over under GAME OVER / YOU SURVIVED N ROUNDS. They replace
//! Black Ops II's scoreboard menu and the scripts' Game Over text (both are
//! left out while the shell is on: `board_ours`).
//!
//! It reads the game's own scoreboard (`bo2zm_scoreboard`: `client:name:
//! score:kills:downs:revives:headshots` by `;`) and round (`bo2zm_round`).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use net::{LocalPresentClient, PresentedSnapshot};

use super::theme::*;
use super::{bridge, window_scale};

/// The shell draws Zombies' scoreboard and Game Over (BO2's are left out).
pub(crate) fn board_ours() -> bool {
    bridge::enabled()
}

/// A script HUD text that is the game over's (left out for ours).
pub(crate) fn is_game_over_text(text: &str) -> bool {
    let t = text.trim().to_ascii_lowercase();
    // (The text, or its localization key: `ZOMBIE_GAME_OVER`,
    // `ZOMBIE_SURVIVED_ROUNDS`.)
    t == "game over" || t.starts_with("you survived") || t.contains("zombie_game_over") || t.contains("zombie_survived_round")
}

#[derive(Component)]
pub(crate) struct BoardRoot;

/// The players' colours, as Black Ops gives the four survivors.
const PLAYER: [Color; 4] = [
    Color::srgb(0.95, 0.95, 0.95),
    Color::srgb(0.35, 0.62, 1.0),
    Color::srgb(0.98, 0.80, 0.25),
    Color::srgb(0.40, 0.85, 0.40),
];

#[derive(Clone, Debug, Default, PartialEq)]
struct Row {
    client: u32,
    name: String,
    cells: [u32; 5],
}

fn rows(board: &str) -> Vec<Row> {
    board
        .split(';')
        .filter(|r| !r.is_empty())
        .filter_map(|r| {
            let f: Vec<&str> = r.split(':').collect();
            let n = |i: usize| f.get(i).and_then(|c| c.trim().parse().ok()).unwrap_or(0);
            Some(Row { client: f.first()?.trim().parse().ok()?, name: (*f.get(1)?).to_owned(), cells: [n(2), n(3), n(4), n(5), n(6)] })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn zm_board(
    mut commands: Commands,
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    actions: Option<Res<net::ClientActionInput>>,
    identity: Option<Res<frame::LaunchIdentity>>,
    fonts: Option<Res<ShellFonts>>,
    roots: Query<Entity, With<BoardRoot>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut drawn: Local<String>,
    mut over_since: Local<Option<f64>>,
) {
    let zm = identity.as_deref().is_some_and(|i| crate::lui_hud::is_zombies_zone(&i.zone));
    let found = match (zm && board_ours(), presented.as_deref(), local.as_deref(), fonts.as_deref()) {
        (true, Some(p), Some(l), Some(f)) => p.snapshot().map(|snap| {
            let d = snap.meta.script_dvars(l.0);
            let round: u32 = d.string("bo2zm_round").and_then(|r| r.trim().parse().ok()).unwrap_or(0);
            let board = d.string("bo2zm_scoreboard").unwrap_or_default().to_owned();
            let over = p.player(l.0).is_some_and(|ps| ps.pm_type == playerstate_iw4::PM_TYPE_INTERMISSION);
            (round, board, over, l.0.0, f.clone())
        }),
        _ => None,
    };
    let now = time.elapsed_secs_f64();
    let held = actions.as_deref().is_some_and(|a| a.client.kb.scores.active);
    let Some((round, board, over, me, fonts)) = found.filter(|(_, _, over, ..)| *over || held) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        drawn.clear();
        *over_since = None;
        return;
    };
    if over && over_since.is_none() {
        *over_since = Some(now);
    }
    // (The game over fades in over its first second.)
    let fade = over_since.map_or(1.0, |t| ((now - t) / 0.8).clamp(0.0, 1.0) as f32);
    let (area, s) = window_scale(&windows);
    let key = format!("{round}|{board}|{over}|{area:?}|{:.2}", fade);
    if *drawn == key && !roots.is_empty() {
        return;
    }
    *drawn = key;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let pal = Palette::ZOMBIES;
    let px = |v: f32| Val::Px(v * s);
    let bold = |v: f32| font(&fonts.bold, v * s);
    let regular = |v: f32| font(&fonts.regular, v * s);
    let (title, mode) = super::zombies::loading().map_or_else(|| (String::new(), String::new()), |z| (z.title, z.mode));
    let list = rows(&board);
    let a = |c: Color| c.with_alpha(c.alpha() * fade);
    commands
        .spawn((
            BoardRoot,
            GlobalZIndex(14_000),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::top(px(if over { 150.0 } else { 220.0 })),
                row_gap: px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, if over { 0.55 * fade } else { 0.25 })),
        ))
        .with_children(|r| {
            if over {
                r.spawn((Text::new("GAME OVER"), font(&fonts.black, 120.0 * s), TextColor(a(WHITE)), TextShadow { offset: Vec2::splat(3.0 * s), color: Color::srgba(0.0, 0.0, 0.0, 0.8 * fade) }));
                let rounds = if round == 1 { "1 ROUND".to_owned() } else { format!("{round} ROUNDS") };
                r.spawn((Text::new(format!("YOU SURVIVED {rounds}")), bold(38.0), TextColor(a(pal.accent)), Node { margin: UiRect::bottom(px(28.0)), ..default() }));
            }
            // The table: the map and mode, then its columns.
            r.spawn((
                Node { width: px(1180.0), flex_direction: FlexDirection::Column, border: UiRect::top(px(3.0)), ..default() },
                BackgroundColor(a(pal.panel)),
                BorderColor::all(a(pal.accent)),
            ))
            .with_children(|t| {
                t.spawn(Node { height: px(64.0), padding: UiRect::horizontal(px(24.0)), align_items: AlignItems::Center, ..default() }).with_children(|h| {
                    h.spawn(Node { flex_grow: 1.0, column_gap: px(14.0), align_items: AlignItems::Baseline, ..default() }).with_children(|n| {
                        n.spawn((Text::new(title.clone()), bold(32.0), TextColor(a(WHITE))));
                        n.spawn((Text::new(mode.clone()), bold(22.0), TextColor(a(pal.accent))));
                    });
                    for col in ["POINTS", "KILLS", "DOWNS", "REVIVES", "HEADSHOTS"] {
                        h.spawn((Node { width: px(136.0), justify_content: JustifyContent::Center, ..default() },))
                            .with_child((Text::new(col), bold(19.0), TextColor(a(DIM))));
                    }
                });
                // Four places, as Black Ops shows them (the empty ones dim).
                for i in 0..4usize {
                    let row = list.get(i);
                    let mine = row.is_some_and(|r| r.client == me);
                    let colour = PLAYER[i];
                    t.spawn((
                        Node { height: px(58.0), padding: UiRect::horizontal(px(24.0)), align_items: AlignItems::Center, border: UiRect::left(px(5.0)), ..default() },
                        BackgroundColor(a(if mine { pal.accent_a(0.20) } else if i % 2 == 0 { pal.panel_soft } else { Color::NONE })),
                        BorderColor::all(a(if row.is_some() { colour } else { Color::NONE })),
                    ))
                    .with_children(|line| {
                        match row {
                            Some(r) => {
                                line.spawn(Node { flex_grow: 1.0, ..default() })
                                    .with_child((Text::new(r.name.clone()), bold(28.0), TextColor(a(colour))));
                                for (k, v) in r.cells.iter().enumerate() {
                                    line.spawn((Node { width: px(136.0), justify_content: JustifyContent::Center, ..default() },))
                                        .with_child((Text::new(v.to_string()), if k == 0 { bold(28.0) } else { regular(27.0) }, TextColor(a(if k == 0 { WHITE } else { TEXT }))));
                                }
                            }
                            None => {
                                line.spawn((Text::new("—"), regular(24.0), TextColor(a(FAINT))));
                            }
                        }
                    });
                }
            });
            if over {
                r.spawn((Text::new("Back to the Zombies menu in a moment..."), regular(22.0), TextColor(a(DIM)), Node { margin: UiRect::top(px(18.0)), ..default() }));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_board_reads_its_rows() {
        let r = rows("0:sero:500:12:1:0:4;3:bot:90:2:0:1:0;");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], Row { client: 0, name: "sero".to_owned(), cells: [500, 12, 1, 0, 4] });
        assert_eq!(r[1].client, 3);
    }

    #[test]
    fn the_game_over_text_is_known() {
        assert!(is_game_over_text("GAME OVER"));
        assert!(is_game_over_text("You Survived 12 Rounds"));
        assert!(!is_game_over_text("Max Ammo!"));
    }
}
