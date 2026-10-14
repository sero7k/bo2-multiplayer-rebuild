//! Infinite Warfare's 2-D pictures by image name (the map previews
//! `image_mp_map_<map>`, the Zombies posters `zm_theater_poster_<map>`), as
//! RGBA, from `ui.ff` in the game's folder (streamed levels from its
//! `imagefile<N>.pak`s).
//!
//! The first call walks `ui.ff` once and keeps the image records (not the
//! pixels); each picture is read when asked for.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::texture;
use crate::walk::ImageRec;

/// The largest level read.
const MAX_DIM: u32 = 1024;

struct Ui {
    images: HashMap<String, (ImageRec, Option<Vec<u8>>)>,
}

fn store() -> &'static Mutex<HashMap<PathBuf, Option<std::sync::Arc<Ui>>>> {
    static STORE: OnceLock<Mutex<HashMap<PathBuf, Option<std::sync::Arc<Ui>>>>> = OnceLock::new();
    STORE.get_or_init(Default::default)
}

fn ui(dir: &Path) -> Option<std::sync::Arc<Ui>> {
    let mut guard = store().lock().ok()?;
    guard
        .entry(dir.to_owned())
        .or_insert_with(|| {
            // (`iw7_ui.ff`: a copy beside the samples.)
            let z = crate::Zone::open(&dir.join("ui.ff")).or_else(|_| crate::Zone::open(&dir.join("iw7_ui.ff"))).ok()?;
            let (cap, _) = crate::walk::walk_zone(&z);
            let images = cap
                .images
                .into_iter()
                .filter(|im| !im.name.starts_with(','))
                .map(|im| {
                    let px = im.pixels.map(|s| s.get(&z.image).to_vec());
                    (im.name.clone(), (im, px))
                })
                .collect();
            Some(std::sync::Arc::new(Ui { images }))
        })
        .clone()
}

/// The first of `names` that `ui.ff` in `dir` holds, decoded: (width,
/// height, RGBA).
pub fn read_ui_image(dir: &Path, names: &[String]) -> Option<(u32, u32, Vec<u8>)> {
    let ui = ui(dir)?;
    names.iter().find_map(|n| {
        let (im, px) = ui.images.get(n.as_str())?;
        let levels = match px {
            Some(data) => texture::ImageLevels {
                format: im.format,
                width: u32::from(im.width),
                height: u32::from(im.height),
                levels: u32::from(im.level_count.max(1)),
                data: data.clone(),
                stream: None,
            },
            None => {
                let k = crate::map::pick_stream(im, MAX_DIM)?;
                let f = im.stream_files[k]?;
                let raw = crate::map::read_image_range(dir, &f).ok()?;
                let part = texture::stream_part(im, k, &raw).ok()?;
                let mut parts = vec![Vec::new(); k + 1];
                parts[k] = part;
                texture::stream_levels(im, k, &parts)
            }
        };
        texture::level_rgba(&levels, 0)
    })
}
