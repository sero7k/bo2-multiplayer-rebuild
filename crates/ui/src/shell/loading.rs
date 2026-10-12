//! bo2mp shell: the loading screen - the map's picture over the whole
//! screen, drifting slowly closer; a black band at the top with the map's
//! name in blue and the mode in white, a tip beside them; a black band at
//! the bottom with what is loading and the bar. It shares the front end's
//! picture, fade and type, and replaces BO2's own loading menu.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::pics::{ShellPics, cover};
use super::theme::*;
use super::{bridge, window_scale};

#[derive(Component)]
pub(crate) struct LoadRoot;

#[derive(Component)]
pub(crate) struct LoadPicture;

#[derive(Component)]
pub(crate) struct LoadBar;

#[derive(Component)]
pub(crate) struct LoadStage;

#[derive(Component)]
pub(crate) struct LoadPercent;

const TIPS: &[&str] = &[
    "Bots follow the map's own paths: hold a choke point and they will come to you.",
    "Every level past 55 still counts toward your next prestige.",
    "Prestige icons come from Black Ops, Modern Warfare 2 and Modern Warfare 3 in turn.",
    "Change the bots' difficulty in the lobby's SETTINGS tab.",
    "Press F10 in the menus to tell the game where your other games are installed.",
    "Objectives only appear in the modes that use them.",
    "Classic maps play by Black Ops II's rules: its weapons, scorestreaks and spawns.",
    "A controller's buttons show in every prompt as soon as you touch it.",
    "Experience comes in three times faster than in the original game.",
];

#[derive(Default)]
pub(crate) struct LoadState {
    request: Option<u64>,
    shown: f32,
    target: f32,
    tip: usize,
    since: f64,
    key: String,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn shell_loading(
    mut commands: Commands,
    loading: Option<Res<assets::LoadingScreen>>,
    setup: Option<Res<session::Bo2mpMatchSetup>>,
    front: Option<Res<crate::lui_frontend::Bo2mpFrontend>>,
    fonts: Option<Res<ShellFonts>>,
    mut pics: ResMut<ShellPics>,
    icons: Option<Res<assets::T6HudIcons>>,
    mut images: ResMut<Assets<Image>>,
    roots: Query<Entity, With<LoadRoot>>,
    mut pictures: Query<(&mut ImageNode, &mut Node), (With<LoadPicture>, Without<LoadBar>)>,
    mut bars: Query<&mut Node, (With<LoadBar>, Without<LoadPicture>)>,
    mut stages: Query<&mut Text, (With<LoadStage>, Without<LoadPercent>)>,
    mut percents: Query<&mut Text, (With<LoadPercent>, Without<LoadStage>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut st: Local<LoadState>,
) {
    let on = bridge::enabled() && front.is_none() && loading.is_some() && setup.is_some();
    bridge::set_loading_shown(on);
    let (Some(loading), Some(setup), Some(fonts), true) = (loading, setup, fonts, on) else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        st.key.clear();
        st.request = None;
        return;
    };
    let now = time.elapsed_secs_f64();
    let snap = loading.progress.snapshot();
    if st.request != Some(snap.request_id) {
        st.request = Some(snap.request_id);
        st.shown = 0.0;
        st.target = 0.0;
        st.since = now;
        st.tip = (snap.request_id as usize).wrapping_mul(7) % TIPS.len();
    }
    // How far: finished stages, and the share of the running ones' counts.
    let total = snap.stages.len().max(1) as f32;
    let done: f32 = snap
        .stages
        .iter()
        .map(|s| {
            if s.end.is_some() {
                1.0
            } else {
                s.count.and_then(|c| c.total.filter(|t| *t > 0).map(|t| c.completed as f32 / t as f32)).unwrap_or(0.0).min(0.95)
            }
        })
        .sum();
    // (Stages join as the load goes, so the bar never runs back, and it
    // keeps a little for the world to appear.)
    st.target = st.target.max((done / total).min(0.96));
    let dt = time.delta_secs().min(0.1);
    st.shown += (st.target - st.shown) * (dt * 3.0).min(1.0);
    let stage = snap
        .stages
        .iter()
        .rev()
        .find(|s| s.end.is_none() && s.started_at.is_some())
        .map_or_else(|| "Preparing the world".to_owned(), |s| format!("Loading {}", s.key.label().to_lowercase()));
    let cat = bridge::catalog();
    let entry = cat.as_ref().and_then(|c| c.map(&setup.map).cloned());
    let map_title = entry.as_ref().map_or_else(|| setup.map.trim_start_matches("mp_").to_uppercase(), |m| m.title.clone());
    let mode_title = cat
        .as_ref()
        .and_then(|c| c.mode(&setup.gametype).map(|m| m.title.clone()))
        .unwrap_or_else(|| setup.gametype.to_uppercase());
    let origin = entry.as_ref().map(|m| m.origin.to_uppercase()).unwrap_or_default();
    // The map's loading picture first, then its menu pictures.
    let mut names = vec![format!("loadscreen_{}", setup.map)];
    if let Some(e) = &entry {
        names.extend(e.pictures.iter().cloned());
    }
    let picture = pics.first(&names, icons.as_deref(), &mut images);
    let (area, s) = window_scale(&windows);
    let key = format!("{map_title}|{mode_title}|{area:?}|{}|{}", picture.is_some(), st.tip);
    if st.key != key || roots.is_empty() {
        st.key = key;
        for e in &roots {
            commands.entity(e).despawn();
        }
        let px = |v: f32| Val::Px(v * s);
        let bold = |v: f32| font(&fonts.bold, v * s);
        let regular = |v: f32| font(&fonts.regular, v * s);
        commands
            .spawn((
                LoadRoot,
                GlobalZIndex(10_500),
                BackgroundColor(Color::srgb(0.012, 0.02, 0.04)),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ))
            .with_children(|r| {
                if let Some(h) = picture.clone() {
                    r.spawn((LoadPicture, ImageNode::new(h), Node { position_type: PositionType::Absolute, ..default() }));
                }
                // A soft vignette, darker toward the bands.
                r.spawn((
                    Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                    BackgroundGradient::from(LinearGradient::to_bottom(vec![
                        ColorStop::new(Color::srgba(0.0, 0.01, 0.03, 0.55), Val::Percent(0.0)),
                        ColorStop::new(Color::srgba(0.0, 0.01, 0.03, 0.0), Val::Percent(22.0)),
                        ColorStop::new(Color::srgba(0.0, 0.01, 0.03, 0.0), Val::Percent(70.0)),
                        ColorStop::new(Color::srgba(0.0, 0.01, 0.03, 0.75), Val::Percent(100.0)),
                    ])),
                ));
                // The top band: map, mode, a tip.
                r.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        height: px(72.0),
                        padding: UiRect::horizontal(px(40.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: px(40.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.88)),
                ))
                .with_children(|top| {
                    top.spawn(Node { column_gap: px(16.0), align_items: AlignItems::Baseline, ..default() }).with_children(|t| {
                        t.spawn((Text::new(map_title.clone()), bold(42.0), TextColor(ACCENT)));
                        t.spawn((Text::new(mode_title.clone()), bold(42.0), TextColor(WHITE)));
                    });
                    top.spawn((Node { max_width: Val::Percent(52.0), ..default() },))
                        .with_child((Text::new(TIPS[st.tip]), regular(21.0), TextColor(DIM)));
                });
                // Where the map is from, over the bottom band.
                if !origin.is_empty() {
                    r.spawn((
                        Node { position_type: PositionType::Absolute, left: px(40.0), bottom: px(84.0), ..default() },
                        Text::new(origin.clone()),
                        bold(22.0),
                        TextColor(TEXT),
                    ));
                }
                // The bottom band: what is loading, the bar, how far.
                r.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        height: px(56.0),
                        padding: UiRect::horizontal(px(40.0)),
                        align_items: AlignItems::Center,
                        column_gap: px(28.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
                ))
                .with_children(|b| {
                    b.spawn((Node { width: px(300.0), ..default() },)).with_child((LoadStage, Text::new(stage.clone()), regular(21.0), TextColor(TEXT)));
                    b.spawn((Node { flex_grow: 1.0, height: px(6.0), ..default() }, BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12))))
                        .with_child((LoadBar, Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(ACCENT)));
                    b.spawn((LoadPercent, Text::new("0%"), bold(21.0), TextColor(WHITE)));
                });
            });
        return;
    }
    // Every frame: the picture's drift, the bar, the stage.
    if let Some(h) = picture
        && let Ok((mut img, mut node)) = pictures.single_mut()
    {
        let size = images.get(&h).map_or(area, |i| i.size_f32());
        let drift = 1.0 + 0.08 * (((now - st.since) / 40.0).clamp(0.0, 1.0) as f32);
        let c = cover(size, area) * drift;
        node.width = Val::Px(c.x);
        node.height = Val::Px(c.y);
        node.left = Val::Px((area.x - c.x) * 0.5);
        node.top = Val::Px((area.y - c.y) * 0.5);
        img.color = Color::srgba(1.0, 1.0, 1.0, (((now - st.since) / 0.8).clamp(0.0, 1.0)) as f32);
    }
    for mut bar in &mut bars {
        bar.width = Val::Percent(st.shown * 100.0);
    }
    for mut t in &mut stages {
        if t.0 != stage {
            t.0.clone_from(&stage);
        }
    }
    let pct = format!("{}%", (st.shown * 100.0).round() as i32);
    for mut t in &mut percents {
        if t.0 != pct {
            t.0 = pct.clone();
        }
    }
}
