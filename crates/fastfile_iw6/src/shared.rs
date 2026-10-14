//! Pictures and materials a map zone names but does not hold.
//!
//! A Ghosts map's material can name an image another zone loads: the
//! image's name then starts with `,` (the linker's "use the loaded one")
//! and its record is empty (format 0, 0 x 0, no pixels, no stream parts).
//! Most are `common_mp`'s (shared floors, walls and trims: Chasm's
//! `ch_floor_broken02_*` cover 1.5 % of its triangles), a few are the
//! engine's own (`$white`, `$black`, `$identitynormalmap`), which no zone
//! holds. A placed model can name a whole material the same way
//! (`,m/mtl_sz_crate_federation`: no technique set, no textures here):
//! Strikezone's and Zebra's crates are `common_core_mp`'s, Sovereign's
//! glass `common_mp`'s.
//!
//! `SharedImages` reads zones once and keeps every image they hold by name
//! (the records, whose stream parts point into the same
//! `imagefile<N>.pak`s, and the bytes of those whose pixels are inline),
//! and every material they hold by name (its technique set's name and its
//! textures by image name).

use std::collections::HashMap;
use std::path::Path;

use crate::image::{ImagePaks, ImagePixels, image_pixels};
use crate::walk::{ImageRec, Span};

/// A material another zone holds, its images by name.
#[derive(Clone, Debug, Default)]
pub struct SharedMaterial {
    pub name: String,
    pub techset: String,
    pub sort_key: u8,
    /// (name hash, sampler state, semantic, image name)
    pub textures: Vec<(u32, u8, u8, String)>,
    pub constants: Vec<crate::walk::MaterialConstant>,
}

/// The images and materials some zones hold, by name (lower case, no
/// leading `,`).
#[derive(Default)]
pub struct SharedImages {
    by_name: HashMap<String, (ImageRec, Vec<u8>)>,
    materials: HashMap<String, SharedMaterial>,
    /// The zones read, for reports.
    pub zones: Vec<String>,
}

/// An image name as a key: lower case, the reference mark dropped.
pub fn image_key(name: &str) -> String {
    name.trim_start_matches(',').to_ascii_lowercase()
}

/// Whether an image record only names an image held elsewhere.
pub fn is_reference(rec: &ImageRec) -> bool {
    rec.format == 0 && rec.pixels.is_none() && !rec.streamed
}

impl SharedImages {
    /// Adds every image `zone` holds (an image already known keeps its
    /// first record: read a patch zone before the zone it patches).
    pub fn add_zone(&mut self, path: &Path) -> Result<usize, String> {
        let zone = crate::Zone::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (cap, _) = crate::walk::walk_zone(&zone);
        let mut added = 0usize;
        for m in &cap.materials {
            let Some(ts) = m.techset else { continue };
            let key = image_key(&m.name);
            if self.materials.contains_key(&key) {
                continue;
            }
            let textures = m
                .textures
                .iter()
                .filter_map(|t| {
                    let img = &cap.images.get(t.image?.index as usize)?.name;
                    Some((t.name_hash, t.sampler_state, t.semantic, img.clone()))
                })
                .collect();
            self.materials.insert(
                key,
                SharedMaterial {
                    name: m.name.trim_start_matches(',').to_owned(),
                    techset: cap.name_of(ts).trim_start_matches(',').to_owned(),
                    sort_key: m.sort_key,
                    textures,
                    constants: m.constants.clone(),
                },
            );
        }
        for rec in &cap.images {
            if is_reference(rec) {
                continue;
            }
            let key = image_key(&rec.name);
            if self.by_name.contains_key(&key) {
                continue;
            }
            let mut rec = rec.clone();
            let bytes = match rec.pixels {
                Some(span) => {
                    let b = span.get(&zone.image).to_vec();
                    rec.pixels = Some(Span {
                        pos: 0,
                        len: b.len(),
                    });
                    b
                }
                None => Vec::new(),
            };
            self.by_name.insert(key, (rec, bytes));
            added += 1;
        }
        self.zones.push(
            path.file_stem()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned()),
        );
        Ok(added)
    }

    /// Bytes kept of inline pixels.
    pub fn inline_bytes(&self) -> usize {
        self.by_name.values().map(|(_, b)| b.len()).sum()
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// A material by name (`,name` or `name`).
    pub fn material(&self, name: &str) -> Option<&SharedMaterial> {
        self.materials.get(&image_key(name))
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// The record of an image by name (`,name` or `name`).
    pub fn get(&self, name: &str) -> Option<&ImageRec> {
        self.by_name.get(&image_key(name)).map(|(r, _)| r)
    }

    /// An image's pixels by name, as `image::image_pixels` reads them
    /// (streamed parts from the paks in `paks`' folder).
    pub fn pixels(
        &self,
        name: &str,
        paks: &mut ImagePaks,
        max_size: u32,
    ) -> Result<ImagePixels, String> {
        let (rec, bytes) = self.by_name.get(&image_key(name)).ok_or_else(|| {
            format!(
                "{} is in none of {:?}",
                name.trim_start_matches(','),
                self.zones
            )
        })?;
        image_pixels(rec, bytes, paks, max_size)
    }
}

/// The engine's built-in pictures a material can name (`$white`, `$black`,
/// `$identitynormalmap`, ...): 4 x 4 `R8G8B8A8_UNORM`, one level, or `None`
/// for a name that is not one of them.
pub fn builtin_pixels(name: &str) -> Option<ImagePixels> {
    let texel: [u8; 4] = match image_key(name).as_str() {
        "$white" => [255, 255, 255, 255],
        "$black" => [0, 0, 0, 255],
        "$gray" | "$grey" => [128, 128, 128, 255],
        // Flat: x and y at the middle, z up.
        "$identitynormalmap" => [128, 128, 255, 255],
        _ => return None,
    };
    Some(ImagePixels {
        width: 4,
        height: 4,
        levels: 1,
        dxgi_format: 28,
        data: texel.repeat(16),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn keys_drop_the_reference_mark() {
        assert_eq!(
            super::image_key(",CH_Floor_Broken02_col"),
            "ch_floor_broken02_col"
        );
        assert!(
            super::builtin_pixels(",$white")
                .is_some_and(|p| p.data.len() == 64 && p.data[0] == 255)
        );
        assert!(super::builtin_pixels("ch_floor").is_none());
    }
}
