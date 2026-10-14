//! bo2mp: Black Ops III pictures for the menus (its weapons' icons, for
//! Create-a-Class): every image of a zone whose name a filter keeps,
//! decoded to RGBA8 at most `max_dim` on a side (pixels from the zone or
//! the `.xpak` packages beside it).

use std::path::Path;

use crate::texture::{image_levels_max, level_rgba};
use crate::walk::{Options, walk_with};
use crate::xpak::XpakSet;

/// One picture: (name, width, height, RGBA8).
pub type Picture = (String, u32, u32, Vec<u8>);

/// The images of the zone at `zone_path` whose names `keep` says yes to,
/// decoded (the ones that do not decode are left out). The walk goes as far
/// as it reads.
pub fn read_images(zone_path: &Path, keep: impl Fn(&str) -> bool, max_dim: u32) -> Result<Vec<Picture>, String> {
    let zone = crate::Zone::open(zone_path).map_err(|e| format!("{}: {e}", zone_path.display()))?;
    let w = walk_with(&zone, Options::default());
    let dir = zone_path.parent().unwrap_or(Path::new("."));
    let paks = XpakSet::open_dir(dir).map(|(p, _)| p).unwrap_or_default();
    let mut out = Vec::new();
    for im in w.capture.images.iter().filter(|im| !im.name.starts_with(',') && keep(&im.name)) {
        let Ok(levels) = image_levels_max(im, &zone.image, &paks, max_dim) else { continue };
        if let Some((width, height, rgba)) = level_rgba(&levels, 0) {
            out.push((im.name.clone(), width, height, rgba));
        }
    }
    Ok(out)
}
