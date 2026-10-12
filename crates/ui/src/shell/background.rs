//! bo2mp shell: the backdrop - a map picture over the whole screen that
//! fades into dark navy on the left (where the menus are), at the top and
//! at the bottom, so the map shows without fighting the text. In the lobby
//! it is the lobby's map, in the map picker the map in focus; on the main
//! and Multiplayer screens the maps take turns. A new picture fades in over
//! the old, and each drifts slowly closer while it is up.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::pics::{ShellPics, cover};
use super::{Screen, Shell, bridge, pages};

#[derive(Component)]
pub(crate) struct BgRoot;

/// 0: the picture now, 1: the one fading out.
#[derive(Component)]
pub(crate) struct BgLayer(u8);

#[derive(Default)]
pub(crate) struct BgState {
    now: Option<(String, Handle<Image>, f64)>,
    old: Option<(Handle<Image>, f64)>,
    slide: usize,
    slide_at: f64,
}

const FADE: f64 = 0.9;
const SLIDE_SECS: f64 = 9.0;

fn spawn(commands: &mut Commands) {
    let full = || Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    };
    commands
        .spawn((
            BgRoot,
            GlobalZIndex(5),
            BackgroundColor(Color::srgb(0.012, 0.02, 0.04)),
            Node {
                overflow: Overflow::clip(),
                ..full()
            },
        ))
        .with_children(|c| {
            for i in [1u8, 0] {
                c.spawn((BgLayer(i), ImageNode::default(), Visibility::Hidden, Node {
                    position_type: PositionType::Absolute,
                    ..default()
                }));
            }
            // The fades: the left side (the menus), the top and the bottom,
            // and a cool tint over it all.
            c.spawn((
                full(),
                BackgroundGradient::from(LinearGradient::to_right(vec![
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.97), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.78), Val::Percent(42.0)),
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.30), Val::Percent(100.0)),
                ])),
            ));
            c.spawn((
                full(),
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.80), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.0), Val::Percent(24.0)),
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.0), Val::Percent(62.0)),
                    ColorStop::new(Color::srgba(0.008, 0.016, 0.035, 0.92), Val::Percent(100.0)),
                ])),
            ));
            c.spawn((full(), BackgroundColor(Color::srgba(0.02, 0.06, 0.14, 0.30))));
        });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn shell_background(
    mut commands: Commands,
    shell: Res<Shell>,
    mut pics: ResMut<ShellPics>,
    icons: Option<Res<assets::T6HudIcons>>,
    mut images: ResMut<Assets<Image>>,
    roots: Query<Entity, With<BgRoot>>,
    mut layers: Query<(&BgLayer, &mut ImageNode, &mut Node, &mut Visibility)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut st: Local<BgState>,
) {
    let up = bridge::covers_lui();
    if !up {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    }
    if roots.is_empty() {
        spawn(&mut commands);
        return;
    }
    let Some(c) = bridge::catalog() else { return };
    let now = time.elapsed_secs_f64();
    // The map to show.
    let target: Option<String> = match shell.screen {
        Screen::Lobby | Screen::ModePick => Some(shell.map.clone()),
        Screen::MapPick => pages(&c)
            .get(shell.page)
            .and_then(|(_, maps)| maps.get(shell.focus))
            .map(|&i| c.maps[i].ui_name.clone()),
        Screen::Main | Screen::Multiplayer => {
            if c.maps.is_empty() {
                None
            } else {
                if now >= st.slide_at {
                    st.slide = (st.slide + 1) % c.maps.len();
                    st.slide_at = now + SLIDE_SECS;
                }
                Some(c.maps[st.slide % c.maps.len()].ui_name.clone())
            }
        }
    };
    if let Some(name) = target
        && st.now.as_ref().is_none_or(|(n, ..)| *n != name)
    {
        let pictures = c.map(&name).map(|m| m.pictures.clone()).unwrap_or_default();
        match pics.first(&pictures, icons.as_deref(), &mut images) {
            Some(h) => {
                st.old = st.now.take().map(|(_, h, _)| (h, now));
                st.now = Some((name, h, now));
            }
            // (A slideshow map with no picture yet: the next one soon.)
            None if matches!(shell.screen, Screen::Main | Screen::Multiplayer) && st.slide_at - now > 0.5 => {
                st.slide_at = now + 0.05;
            }
            None => {}
        }
    }
    let (area, _) = super::window_scale(&windows);
    for (layer, mut img, mut node, mut vis) in &mut layers {
        let pic = match layer.0 {
            0 => st.now.as_ref().map(|(_, h, at)| (h.clone(), *at, ((now - at) / FADE).clamp(0.0, 1.0))),
            _ => st.old.as_ref().map(|(h, at)| (h.clone(), *at, 1.0 - ((now - at) / FADE).clamp(0.0, 1.0))),
        };
        let Some((h, since, alpha)) = pic.filter(|p| p.2 > 0.001) else {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
            continue;
        };
        if img.image != h {
            img.image = h.clone();
        }
        img.color = Color::srgba(1.0, 1.0, 1.0, alpha as f32);
        // Covering the screen, drifting closer over 30 seconds.
        let size = images.get(&h).map_or(area, |i| i.size_f32());
        let drift = 1.0 + 0.06 * ((now - since) / 30.0).clamp(0.0, 1.0) as f32;
        let s = cover(size, area) * drift;
        node.width = Val::Px(s.x);
        node.height = Val::Px(s.y);
        node.left = Val::Px((area.x - s.x) * 0.5);
        node.top = Val::Px((area.y - s.y) * 0.5);
        if *vis != Visibility::Inherited {
            *vis = Visibility::Inherited;
        }
    }
    if st.old.as_ref().is_some_and(|(_, at)| now - at > FADE) {
        st.old = None;
    }
}
