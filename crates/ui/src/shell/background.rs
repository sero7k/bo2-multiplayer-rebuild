//! bo2mp shell: the backdrop - a map picture over the whole screen that
//! fades into dark navy on the left (where the menus are), at the top and
//! at the bottom, so the map shows without fighting the text. In the lobby
//! it is the lobby's map, in the map picker the map in focus; on the main
//! and Multiplayer screens the maps take turns. A new picture fades in over
//! the old, and each drifts slowly closer while it is up. The Zombies
//! screens show the Zombies map (and start) in the same way, faded into a
//! dark red (`theme::palette`).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::pics::{ShellPics, cover};
use super::theme::{Palette, palette};
use super::{Screen, Shell, bridge, pages, zombies};

#[derive(Component)]
pub(crate) struct BgRoot;

/// 0: the picture now, 1: the one fading out.
#[derive(Component)]
pub(crate) struct BgLayer(u8);

#[derive(Default)]
pub(crate) struct BgState {
    now: Option<(String, Handle<Image>, f64)>,
    old: Option<(Handle<Image>, f64)>,
    slide_at: f64,
    seed: u64,
    was_main: bool,
    /// The colours the fades were made in.
    pal: Option<Palette>,
}

const FADE: f64 = 0.9;

/// The menu photographs (the main screens' backdrop while no map is
/// chosen), shown in a random order.
const PHOTOS: [&[u8]; 21] = [
    include_bytes!("../../assets/menu_backgrounds/bg01.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg02.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg03.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg04.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg05.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg06.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg07.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg08.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg09.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg10.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg11.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg12.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg13.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg14.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg15.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg16.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg17.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg18.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg19.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg20.jpg"),
    include_bytes!("../../assets/menu_backgrounds/bg21.jpg"),
];

/// The Zombies menu's survivors (the Zombies screens' backdrop while no
/// map is shown), in a random order.
const ZM_PHOTOS: [&[u8]; 12] = [
    include_bytes!("../../assets/menu_backgrounds/zm01.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm02.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm03.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm04.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm05.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm06.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm07.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm08.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm09.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm10.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm11.jpg"),
    include_bytes!("../../assets/menu_backgrounds/zm12.jpg"),
];

/// A small random number source (the order the photos come in).
fn next_random(seed: &mut u64) -> u64 {
    if *seed == 0 {
        *seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0x9E37_79B9, |d| d.as_nanos() as u64) | 1;
    }
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

/// The photo `i` (of the Zombies set when `zm`) as a picture (decoded once).
fn photo(i: usize, zm: bool, pics: &mut ShellPics, images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let name = format!("\u{1}menu photo {}{i}", if zm { "zm " } else { "" });
    if let Some(h) = pics.get(&name, None, images) {
        return Some(h);
    }
    let set: &[&[u8]] = if zm { &ZM_PHOTOS } else { &PHOTOS };
    let img = Image::from_buffer(
        set.get(i)?,
        bevy::image::ImageType::Extension("jpg"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::linear(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .map_err(|e| diag::warn!(Ui, "bo2mp shell: menu photo {i} did not decode: {e}"))
    .ok()?;
    let h = images.add(img);
    pics.insert(&name, h.clone());
    Some(h)
}

/// The screens with no map to show: the photos take turns there.
fn photo_screen(shell: &Shell) -> bool {
    match shell.screen {
        Screen::Main | Screen::Multiplayer | Screen::Options | Screen::Barracks | Screen::CardPick | Screen::EmblemPick => true,
        Screen::Classes | Screen::ClassEdit | Screen::ItemPick => shell.cac_from != Screen::Lobby,
        Screen::Zombies | Screen::ZmRecord => true,
        _ => false,
    }
}
const SLIDE_SECS: f64 = 9.0;

fn spawn(commands: &mut Commands, pal: Palette) {
    let [r, g, b] = pal.shade;
    let shade = |a: f32| Color::srgba(r, g, b, a);
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
            BackgroundColor(Color::srgb(r * 1.5, g * 1.25, b * 1.15)),
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
                    ColorStop::new(shade(0.97), Val::Percent(0.0)),
                    ColorStop::new(shade(0.78), Val::Percent(42.0)),
                    ColorStop::new(shade(0.30), Val::Percent(100.0)),
                ])),
            ));
            c.spawn((
                full(),
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(shade(0.80), Val::Percent(0.0)),
                    ColorStop::new(shade(0.0), Val::Percent(24.0)),
                    ColorStop::new(shade(0.0), Val::Percent(62.0)),
                    ColorStop::new(shade(0.92), Val::Percent(100.0)),
                ])),
            ));
            c.spawn((full(), BackgroundColor(pal.tint)));
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
    // (New colours: the backdrop is made again; its pictures stay.)
    let pal = palette(shell.screen);
    if roots.is_empty() || st.pal != Some(pal) {
        for e in &roots {
            commands.entity(e).despawn();
        }
        spawn(&mut commands, pal);
        st.pal = Some(pal);
        return;
    }
    let Some(c) = bridge::catalog() else { return };
    let now = time.elapsed_secs_f64();
    // Back on the main menu: a new random photo at once.
    let on_main = shell.screen == Screen::Main;
    if on_main && !st.was_main {
        st.slide_at = 0.0;
    }
    st.was_main = on_main;
    if photo_screen(&shell) {
        // (The Zombies screens show the survivors, the others the soldiers.)
        let zm = shell.screen.is_zombies();
        let (set, tag) = if zm { (ZM_PHOTOS.len(), "zmphoto:") } else { (PHOTOS.len(), "photo:") };
        if now >= st.slide_at || st.now.as_ref().is_none_or(|(n, ..)| !n.starts_with(tag)) {
            // (Never the same photo twice in a row.)
            let mut i = (next_random(&mut st.seed) % set as u64) as usize;
            if st.now.as_ref().is_some_and(|(n, ..)| *n == format!("{tag}{i}")) {
                i = (i + 1) % set;
            }
            st.slide_at = now + SLIDE_SECS;
            if let Some(h) = photo(i, zm, &mut pics, &mut images) {
                st.old = st.now.take().map(|(_, h, _)| (h, now));
                st.now = Some((format!("{tag}{i}"), h, now));
            }
        }
    } else {
        // The map to show.
        // (A Zombies map and start is `zm:<map>:<start>`.)
        let zm = |map: &str, start: usize| Some(format!("zm:{map}:{start}"));
        let target: Option<String> = match shell.screen {
            Screen::MapPick => pages(&c)
                .get(shell.page)
                .and_then(|(_, maps)| maps.get(shell.focus))
                .map(|&i| c.maps[i].ui_name.clone()),
            Screen::ZmMapPick => zombies::pages(zombies::state().bo1())
                .get(shell.page)
                .and_then(|(_, maps)| maps.get(shell.focus))
                .and_then(|&i| zm(zombies::MAPS[i].map, 0)),
            Screen::ZmStartPick => zm(&shell.zm_map, shell.focus),
            Screen::ZmLobby => zm(&shell.zm_map, shell.zm_start),
            _ => Some(shell.map.clone()),
        };
        if let Some(name) = target
            && st.now.as_ref().is_none_or(|(n, ..)| *n != name)
        {
            let pictures = match name.strip_prefix("zm:").and_then(|r| r.rsplit_once(':')) {
                Some((map, start)) => zombies::pictures(zombies::map(map), start.parse().unwrap_or(0)),
                None => c.map(&name).map(|m| m.pictures.clone()).unwrap_or_default(),
            };
            if let Some(h) = pics.first(&pictures, icons.as_deref(), &mut images) {
                st.old = st.now.take().map(|(_, h, _)| (h, now));
                st.now = Some((name, h, now));
            }
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
