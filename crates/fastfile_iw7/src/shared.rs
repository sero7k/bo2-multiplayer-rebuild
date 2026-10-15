//! Pictures and materials a map zone names but does not hold.
//!
//! As in Ghosts (`fastfile_iw6::shared`), an IW7 map's material can name an
//! image another zone loads (`,name`: a record with no pixels and no stream
//! parts) and a placed model can name a whole material another zone holds
//! (`mo/glass_clear_generic_01`, `mo/black_solid`). They are the common
//! zones' (`common_mp`, `common_core_mp`, `common` and their patches, the
//! materials in their `techsets_*` zones), or the engine's own (`$white`,
//! `$black`, `$identitynormalmap`), which no zone holds.
//!
//! `SharedZones` reads zones once and keeps every image they hold by name
//! (its record; the pixels when inline, else the `imagefile<N>.pak` ranges,
//! read when a map wants them) and every material by name (as the map
//! reader's `MapMaterial`). A zone whose walk stops partway (`common_mp`)
//! gives what was read before the stop.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::map::MapMaterial;
use crate::texture::ImageLevels;
use crate::walk::ImageRec;

/// Images and materials some zones hold, by name (lower case, no leading
/// `,`).
#[derive(Default)]
pub struct SharedZones {
    images: HashMap<String, (ImageRec, Vec<u8>)>,
    materials: HashMap<String, MapMaterial>,
    /// Where the `imagefile` paks are.
    pub pak_dir: PathBuf,
    /// The zones read (and how far), for reports.
    pub zones: Vec<String>,
}

impl std::fmt::Debug for SharedZones {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedZones({} images, {} materials from {:?})", self.images.len(), self.materials.len(), self.zones)
    }
}

/// The largest inline picture kept (bytes, all levels): a 512 x 512 BC7
/// picture with its levels is 350 KB.
pub const INLINE_MAX: usize = 400 * 1024;

/// A name as a key: lower case, the reference mark dropped.
pub fn key(name: &str) -> String {
    name.trim_start_matches(',').to_ascii_lowercase()
}

/// The zones a multiplayer map's shared pictures and materials are in,
/// beside it (a patch before the zone it patches): those present.
pub fn common_zones(dir: &Path) -> Vec<PathBuf> {
    [
        "patch_common_mp",
        "common_mp",
        "techsets_common_mp",
        "patch_common_core_mp",
        "common_core_mp",
        "techsets_common_core_mp",
        "patch_common",
        "common",
        "techsets_common",
        // (The glass and the plain materials placed models name:
        // `mo/glass_clear_generic_01`, `mo/lambert1`, `glass_clear_01_c`.)
        "patch_global_core_mp",
        "global_core_mp",
        "techsets_global_core_mp",
    ]
    .iter()
    .map(|z| dir.join(format!("{z}.ff")))
    .filter(|p| p.is_file())
    .collect()
}

impl SharedZones {
    /// Reads `zones` (a name already known keeps its first record).
    pub fn read(zones: &[PathBuf]) -> Self {
        let mut out = SharedZones { pak_dir: zones.first().and_then(|p| p.parent()).map(Path::to_path_buf).unwrap_or_default(), ..Default::default() };
        for z in zones {
            match out.add_zone(z) {
                Ok(line) => out.zones.push(line),
                Err(e) => out.zones.push(e),
            }
        }
        out
    }

    /// Adds what one zone holds; the report line.
    pub fn add_zone(&mut self, path: &Path) -> Result<String, String> {
        let zone = crate::Zone::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (cap, stop) = crate::walk::walk_zone(&zone);
        let (mut images, mut materials, mut bytes_kept, mut too_big) = (0usize, 0usize, 0usize, 0usize);
        for m in &cap.materials {
            let Some(ts) = m.techset else { continue };
            let k = key(&m.name);
            if self.materials.contains_key(&k) {
                continue;
            }
            let textures = m
                .textures
                .iter()
                .filter_map(|t| Some((t.name_hash, t.sampler_state, t.semantic, cap.name_of(t.image?).trim_start_matches(',').to_owned())))
                .collect();
            self.materials.insert(
                k,
                MapMaterial {
                    name: m.name.trim_start_matches(',').to_owned(),
                    technique_set: cap.name_of(ts).trim_start_matches(',').to_owned(),
                    sort_key: m.sort_key,
                    material_type: m.material_type,
                    textures,
                    constants: m.constants.iter().map(|c| (c.name.clone(), c.literal)).collect(),
                },
            );
            materials += 1;
        }
        for im in &cap.images {
            if im.name.starts_with(',') || (im.pixels.is_none() && !im.streamed) {
                continue;
            }
            let k = key(&im.name);
            if self.images.contains_key(&k) {
                continue;
            }
            // (Large inline pictures are the menus' and effects', not a
            // map's surfaces': not kept, as the cache lives with the game.)
            if im.pixels.is_some_and(|p| p.len > INLINE_MAX) {
                too_big += 1;
                continue;
            }
            let mut rec = im.clone();
            let bytes = match rec.pixels {
                Some(span) => {
                    let b = span.get(&zone.image).to_vec();
                    rec.pixels = Some(crate::walk::Span { pos: 0, len: b.len() });
                    b
                }
                None => Vec::new(),
            };
            bytes_kept += bytes.len();
            self.images.insert(k, (rec, bytes));
            images += 1;
        }
        let name = path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().into_owned());
        Ok(match stop {
            Some(s) => format!("{name} ({images} images, {} KB inline, {too_big} large left out, {materials} materials; walk stopped at asset {})", bytes_kept / 1024, s.asset_index),
            None => format!("{name} ({images} images, {} KB inline, {too_big} large left out, {materials} materials)", bytes_kept / 1024),
        })
    }

    pub fn material(&self, name: &str) -> Option<&MapMaterial> {
        self.materials.get(&key(name))
    }

    pub fn image_rec(&self, name: &str) -> Option<&ImageRec> {
        self.images.get(&key(name)).map(|(r, _)| r)
    }

    /// An image's inline pixels (`None` when it is streamed).
    pub fn inline(&self, name: &str) -> Option<&[u8]> {
        self.images.get(&key(name)).filter(|(r, _)| r.pixels.is_some()).map(|(_, b)| b.as_slice())
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }
}

/// The engine's built-in pictures a material can name (`$white`, `$black`,
/// `$identitynormalmap`, ...): 4 x 4 `R8G8B8A8_UNORM`, one level; `None`
/// for a name that is not one of them.
pub fn builtin(name: &str) -> Option<ImageLevels> {
    let texel: [u8; 4] = match key(name).as_str() {
        "$white" => [255, 255, 255, 255],
        "$black" => [0, 0, 0, 255],
        "$gray" | "$grey" => [128, 128, 128, 255],
        "$identitynormalmap" => [128, 128, 255, 255],
        _ => return None,
    };
    Some(ImageLevels { format: 28, width: 4, height: 4, levels: 1, data: texel.repeat(16), stream: None })
}

#[cfg(test)]
mod tests {
    #[test]
    fn builtins_and_keys() {
        assert_eq!(super::key(",Glass_Clear_01_c"), "glass_clear_01_c");
        let w = super::builtin(",$white").unwrap();
        assert_eq!((w.width, w.data.len(), w.data[0]), (4, 64, 255));
        assert!(super::builtin("koth_grid_01_c").is_none());
        let rgba = crate::texture::level_rgba(&super::builtin("$black").unwrap(), 0).unwrap();
        assert_eq!(&rgba.2[..4], &[0, 0, 0, 255]);
    }
}
