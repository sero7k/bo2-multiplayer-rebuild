//! bo2mp shell: the games whose weapons are not read yet - Black Ops Cold
//! War, Vanguard and Modern Warfare Remastered. Each has a tab in
//! Create-a-Class's weapon picker holding one placeholder tile: the game's
//! banner and title, and that its weapons are coming.
//!
//! A game's banner is a picture (`banners\<key>.png` or `.jpg` beside the
//! game, else `banner.png` / `banner.jpg` in the game's folder as set in
//! Game Folders); without one the banner is drawn: the game's title on its
//! colour.

use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;

use super::theme::*;
use super::view::{Ctx, Kids, abs, picture_box};

pub(crate) struct Coming {
    /// Its key in `game_paths.cfg`.
    pub key: &'static str,
    /// Its tab in the weapon picker.
    pub tab: &'static str,
    /// Its title in full.
    pub title: &'static str,
    /// The drawn banner's colour.
    pub tint: Color,
}

pub(crate) const COMING: [Coming; 3] = [
    Coming {
        key: "cw",
        tab: "COLD WAR",
        title: "BLACK OPS COLD WAR",
        tint: Color::srgb(0.42, 0.07, 0.06),
    },
    Coming {
        key: "vg",
        tab: "VANGUARD",
        title: "VANGUARD",
        tint: Color::srgb(0.30, 0.22, 0.10),
    },
    Coming {
        key: "mwr",
        tab: "MWR",
        title: "MODERN WARFARE REMASTERED",
        tint: Color::srgb(0.10, 0.20, 0.12),
    },
];

/// A placeholder pick's value: below every real item's (`PLACEHOLDER - i`
/// for the `i`th game).
const PLACEHOLDER: i32 = -1000;

/// The game a placeholder pick's value stands for.
pub(crate) fn of_value(value: i32) -> Option<&'static Coming> {
    let i = PLACEHOLDER.checked_sub(value)?;
    usize::try_from(i).ok().and_then(|i| COMING.get(i))
}

/// The game whose tab this is.
pub(crate) fn by_tab(tab: &str) -> Option<usize> {
    COMING.iter().position(|g| g.tab == tab)
}

/// The placeholder pick on a game's tab.
pub(crate) fn placeholder(i: usize) -> super::cac::Pick {
    let g = &COMING[i];
    super::cac::Pick {
        value: PLACEHOLDER - i as i32,
        name: "COMING SOON".to_owned(),
        desc: format!(
            "{}'s weapons are coming to Create-a-Class.",
            title_words(g.title)
        ),
        image: banner_name(g.key),
        cost: 0,
        locked: false,
        level: 1,
        group: "COMING SOON".to_owned(),
    }
}

/// `BLACK OPS COLD WAR` -> `Black Ops Cold War` (MWR and the like stay).
fn title_words(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| {
                f.to_uppercase()
                    .chain(c.flat_map(char::to_lowercase))
                    .collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const BANNER: &str = "gamebanner:";

fn banner_name(key: &str) -> String {
    format!("{BANNER}{key}")
}

pub(crate) fn is_banner(name: &str) -> bool {
    name.starts_with(BANNER)
}

/// A game's banner picture, read off the main thread (`None` until it is
/// in, and when there is none).
pub(crate) fn picture(name: &str) -> Option<Arc<Image>> {
    let key = name.strip_prefix(BANNER)?;
    let path = banner_file(key)?;
    crate::foreign_maps::picture_async(name, true, move || read(&path))
}

fn banner_file(key: &str) -> Option<PathBuf> {
    let mut dirs: Vec<(PathBuf, Vec<String>)> = Vec::new();
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(std::path::Path::to_path_buf))
    {
        dirs.push((
            exe_dir.join("banners"),
            vec![
                format!("{key}.png"),
                format!("{key}.jpg"),
                format!("{key}.jpeg"),
            ],
        ));
    }
    if let Some(folder) = asset_transport::game_paths::get(key) {
        dirs.push((
            folder,
            vec!["banner.png".to_owned(), "banner.jpg".to_owned()],
        ));
    }
    dirs.into_iter()
        .flat_map(|(d, names)| names.into_iter().map(move |n| d.join(n)))
        .find(|p| p.is_file())
}

fn read(path: &std::path::Path) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = std::fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let ext = if ext == "jpeg" { "jpg".to_owned() } else { ext };
    let img = Image::from_buffer(
        &bytes,
        bevy::image::ImageType::Extension(&ext),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        bevy::asset::RenderAssetUsages::default(),
    )
    .ok()?;
    let (w, h) = (img.width(), img.height());
    let data = img.data?;
    (data.len() == (w * h * 4) as usize).then_some((w, h, data))
}

/// A game's banner filling `w` x `h`: its picture, else its title drawn on
/// its colour; "WEAPONS COMING SOON" under the title either way.
pub(crate) fn banner(c: &mut Kids, cx: &mut Ctx, g: &Coming, w: f32, h: f32) {
    let drawn = cx.pic(&[banner_name(g.key)]).is_none();
    c.spawn((
        Node {
            width: cx.px(w),
            height: cx.px(h),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(g.tint),
    ))
    .with_children(|b| {
        if !drawn {
            picture_box(b, cx, &[banner_name(g.key)], w, h, ());
        }
        // (A dark band at the bottom so the words read over a picture.)
        b.spawn((
            Node {
                width: Val::Percent(100.0),
                height: cx.px(h * 0.5),
                ..abs(Some(0.0), None, None, Some(0.0), cx)
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, if drawn { 0.25 } else { 0.55 })),
        ));
        let size = if g.title.len() > 18 {
            h * 0.13
        } else {
            h * 0.2
        };
        b.spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: cx.px(4.0),
            ..abs(Some(22.0), None, Some(22.0), Some(18.0), cx)
        },))
            .with_children(|t| {
                t.spawn((Text::new(g.title), cx.black(size), TextColor(WHITE)));
                t.spawn((
                    Text::new("WEAPONS COMING SOON"),
                    cx.bold((h * 0.07).max(14.0)),
                    TextColor(cx.pal.accent),
                ));
            });
    });
}
