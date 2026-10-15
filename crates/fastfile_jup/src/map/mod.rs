//! A Modern Warfare III map read into plain data: the world's surfaces,
//! the entities, the bounds.
//!
//! - [`index`]: the zone image, its asset list, the name hash.
//! - [`world`]: the world's surfaces (`gfx_map` records over the
//!   `gfx_map_trzone` buffers).
//! - [`tiles`]: the streamed world zones (`<map>_<x>_<y>_0_<lod>_tr`, the
//!   swap zones) a big map keeps most of its world in.
//! - [`ents`]: the entities (the install's entity text, else none) and the
//!   spawn points a multiplayer match needs.

pub mod ents;
pub mod index;
pub mod tiles;
pub mod world;

use std::path::{Path, PathBuf};

use crate::oodle::Decompressor;
pub use index::{ZoneImage, name_hash};
pub use world::WorldSurface;

/// Where a map's files are: the install root and the map's name.
#[derive(Clone, Debug)]
pub struct MapSources {
    pub install: PathBuf,
    pub name: String,
}

impl MapSources {
    /// From the map zone's path (`<install>/zone/<map>.ff`).
    pub fn beside(zone: &Path) -> Self {
        let name = zone
            .file_stem()
            .map(|s| s.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let install = zone
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_default();
        Self { install, name }
    }

    pub fn zone(&self, name: &str) -> PathBuf {
        self.install.join("zone").join(format!("{name}.ff"))
    }
}

/// A map, read.
#[derive(Default)]
pub struct JupMap {
    pub name: String,
    pub surfaces: Vec<WorldSurface>,
    /// The entity text (Radiant's `{ "key" "value" }` blocks).
    pub entities: String,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    /// What was read and what was missing, line by line.
    pub report: Vec<String>,
}

/// Reads one zone into its image.
pub fn read_image(path: &Path, oodle: &dyn Decompressor) -> Result<ZoneImage, String> {
    let (_, bytes) =
        crate::container::read_zone(path, oodle).map_err(|e| format!("{}: {e}", path.display()))?;
    ZoneImage::parse(bytes).map_err(|e| format!("{}: {e}", path.display()))
}

impl JupMap {
    /// Reads world zone `k`'s surfaces (its buffers in `z`).
    fn read_zone(
        &mut self,
        name: &str,
        z: &[u8],
        b: &world::WorldBuffers,
        recs: &[world::SurfaceRec],
        k: u32,
    ) -> Result<(), String> {
        let read = world::read_surfaces(z, b, recs, k)?;
        let tris: usize = read.iter().map(|s| s.tris.len()).sum();
        self.report.push(format!(
            "world zone {k} {name}: {} surfaces, {tris} triangles ({:.1} MiB of vertex streams)",
            read.len(),
            b.vertex_bytes as f64 / (1024.0 * 1024.0)
        ));
        self.surfaces.extend(read);
        Ok(())
    }

    pub fn load(src: &MapSources, oodle: &dyn Decompressor) -> Result<Self, String> {
        let mut out = JupMap {
            name: src.name.clone(),
            ..Default::default()
        };
        let started = std::time::Instant::now();
        let zone = read_image(&src.zone(&src.name), oodle)?;
        out.report.push(format!(
            "zone: {} assets, {:.1} MiB, inflated in {:.1}s",
            zone.types.len(),
            zone.bytes.len() as f64 / (1024.0 * 1024.0),
            started.elapsed().as_secs_f32()
        ));
        let z = &zone.bytes;
        let (list, count) =
            world::find_surfaces(z).ok_or("no surface list (gfx_map) in the map zone")?;
        let recs = world::records(z, list, count);
        let mut zones: Vec<u32> = recs.iter().map(|r| r.zone).collect();
        zones.sort_unstable();
        zones.dedup();
        out.report.push(format!(
            "surface list: {count} surfaces over {} world zone(s)",
            zones.len()
        ));
        // The map's own world zone.
        if zones.contains(&0) {
            let buffers = world::find_buffers(z, name_hash(&src.name))
                .ok_or("no world zone (gfx_map_trzone) of the map's name")?;
            out.read_zone(&src.name, z, &buffers, &recs, 0)?;
        }
        drop(zone);
        // The streamed tiles, in the order the surfaces count them.
        let names = tiles::names(src);
        if zones.iter().any(|&k| k > 0) {
            out.report
                .push(format!("tiles: {} streamed world zones", names.len()));
        }
        for &k in zones.iter().filter(|&&k| k > 0) {
            let Some(name) = names.get(k as usize - 1) else {
                out.report
                    .push(format!("tile {k}: no zone (only {} found)", names.len()));
                continue;
            };
            if !tiles::drawn(name) {
                out.report
                    .push(format!("tile {k} {name}: another state, left out"));
                continue;
            }
            let image = match read_image(&src.zone(name), oodle) {
                Ok(i) => i,
                Err(e) => {
                    out.report.push(format!("tile {k} {name}: {e}"));
                    continue;
                }
            };
            let Some(buffers) = world::find_buffers(&image.bytes, name_hash(name)) else {
                out.report
                    .push(format!("tile {k} {name}: no world zone of its name"));
                continue;
            };
            let want = world::packed_of(&recs, k);
            if buffers.packed_bytes != want {
                out.report.push(format!(
                    "tile {k} {name}: {} packed triangle bytes, the surfaces want {want}",
                    buffers.packed_bytes
                ));
                continue;
            }
            if let Err(e) = out.read_zone(name, &image.bytes, &buffers, &recs, k) {
                out.report.push(format!("tile {k} {name}: {e}"));
            }
        }
        if out.surfaces.is_empty() {
            return Err("no world surface read".into());
        }
        let tris: usize = out.surfaces.iter().map(|s| s.tris.len()).sum();
        out.report.push(format!(
            "world: {} surfaces, {} vertices, {tris} triangles",
            out.surfaces.len(),
            out.surfaces
                .iter()
                .map(|s| s.positions.len())
                .sum::<usize>(),
        ));
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for p in out.surfaces.iter().flat_map(|s| s.positions.iter()) {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        (out.mins, out.maxs) = (lo, hi);
        match ents::read(src) {
            Ok(text) => {
                out.entities = text;
                out.report
                    .push(format!("entities: {} bytes", out.entities.len()));
            }
            Err(e) => out.report.push(format!("entities: {e}")),
        }
        Ok(out)
    }
}
