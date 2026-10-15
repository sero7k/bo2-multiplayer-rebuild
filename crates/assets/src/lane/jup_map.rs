//! bo2mp: Call of Duty: Modern Warfare III (2023, `jup`) multiplayer maps.
//!
//! `fastfile_jup::map` reads the map zone (`IWffa100`, inflated through the
//! game's own Oodle) into plain data: the world's surfaces, the entities.
//! This lays them out as a Black Ops III map read (`T7Map`) and builds the
//! match's world through the Black Ops III lane's builder: the world mesh
//! (IW3 builder), collision from the drawn world (no clip map read yet),
//! entities with the spawn points the map needs (Modern Warfare III keeps
//! them out of its entities: `fastfile_jup::map::ents` makes them over the
//! map's path nodes), so Black Ops II's rules play it as an Infinity Ward
//! map.

use std::path::Path;

use asset_material::MaterialCatalog;
use asset_transport::progress::{LoadProgress, StageId};
use asset_world::WorldDrawPolicy;
use fastfile_jup::map::materials::Packages;
use fastfile_jup::map::{JupMap, MapSources, ents};
use fastfile_t7::map::{MapMaterial, T7Map, WorldSurface, WorldVertex};
use fastfile_t7::texture::ImageLevels;

use super::LoadedWorld;
use crate::lane_capability::PreparedCapability;

/// Smooth normals for a triangle list (area-weighted).
fn normals_of(positions: &[[f32; 3]], tris: &[[u16; 3]]) -> Vec<[f32; 3]> {
    let mut n = vec![[0.0f32; 3]; positions.len()];
    for t in tris {
        let [a, b, c] = t.map(|i| positions[usize::from(i)]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let f = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        for &i in t {
            for k in 0..3 {
                n[usize::from(i)][k] += f[k];
            }
        }
    }
    n.into_iter()
        .map(|v| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if l > 1e-12 {
                v.map(|x| x / l)
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect()
}

/// A tangent for a normal (any vector across it).
fn tangent_of(n: [f32; 3]) -> [f32; 3] {
    let a = if n[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let t = [
        a[1] * n[2] - a[2] * n[1],
        a[2] * n[0] - a[0] * n[2],
        a[0] * n[1] - a[1] * n[0],
    ];
    let l = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt().max(1e-6);
    t.map(|x| x / l)
}

/// Where the map is played: a circle (middle, radius) on the ground plan.
#[derive(Clone, Copy, Debug)]
struct PlayArea {
    mid: [f32; 2],
    radius: f32,
}

impl PlayArea {
    /// Does a box (on the ground plan) come within the area?
    fn reaches(&self, lo: [f32; 3], hi: [f32; 3]) -> bool {
        let dx = (self.mid[0] - self.mid[0].clamp(lo[0], hi[0])).abs();
        let dy = (self.mid[1] - self.mid[1].clamp(lo[1], hi[1])).abs();
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

/// A surface too big to stand on as one (a sea sheet, a far vista's
/// ground) spans more than this on the ground plan; it is not drawn.
const SHEET: f32 = 16384.0;

/// Floor points when the map has no path nodes: the middles of the
/// world's upward triangles of a walkable size (not the sea's or a far
/// vista's huge sheets), around where most of them are (a big map's
/// streamed backdrop tiles out), above the lowest few (the sea bed); and
/// the play area they make.
fn floor_points(map: &JupMap) -> (Vec<[f32; 3]>, Option<PlayArea>) {
    let mut pts = Vec::new();
    for s in &map.surfaces {
        for t in &s.tris {
            let [a, b, c] = t.map(|i| s.positions[usize::from(i)]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let f = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let area2 = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
            if area2 > 2.0 * 32.0 * 32.0 && area2 < 2.0 * 512.0 * 512.0 && f[2] / area2 > 0.85 {
                pts.push(std::array::from_fn(|k| (a[k] + b[k] + c[k]) / 3.0));
            }
        }
    }
    if pts.len() < 10 {
        return (pts, None);
    }
    let median = |k: usize, pts: &[[f32; 3]]| {
        let mut v: Vec<f32> = pts.iter().map(|p| p[k]).collect();
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };
    // Around the middle: within 1.5 times the distance six in ten are in.
    let mid = [median(0, &pts), median(1, &pts)];
    let d2 = |p: &[f32; 3]| (p[0] - mid[0]).powi(2) + (p[1] - mid[1]).powi(2);
    let mut d: Vec<f32> = pts.iter().map(d2).collect();
    d.sort_by(f32::total_cmp);
    let reach = d[d.len() * 6 / 10] * 1.5 * 1.5;
    pts.retain(|p| d2(p) <= reach);
    // Above the lowest twentieth.
    let mut z: Vec<f32> = pts.iter().map(|p| p[2]).collect();
    z.sort_by(f32::total_cmp);
    let floor = z[z.len() / 20];
    pts.retain(|p| p[2] > floor);
    let area = PlayArea {
        mid,
        // (A little past the floor's edge: what borders it is solid too.)
        radius: reach.sqrt() * 1.25,
    };
    (pts, Some(area))
}

/// The map as a Black Ops III map read.
fn as_t7(map: &JupMap, report: &mut Vec<String>) -> T7Map {
    let mut t7 = T7Map::default();
    t7.name = map.name.clone();
    let mut sheets = 0usize;
    for s in &map.surfaces {
        if s.tris.is_empty()
            || s.positions.is_empty()
            || s.positions.len() > usize::from(u16::MAX)
            || s.tris.len() > usize::from(u16::MAX)
        {
            continue;
        }
        // A sheet wider than the play area (the sea, the sea bed, a far
        // vista's ground) is left out: it would stretch the world's bounds
        // (and the view's far plane with them) tens of times past the map
        // and lie over it in the depth buffer.
        let extent = |k: usize| {
            let (lo, hi) = s
                .positions
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                    (lo.min(p[k]), hi.max(p[k]))
                });
            hi - lo
        };
        if extent(0) > SHEET || extent(1) > SHEET {
            sheets += 1;
            continue;
        }
        let normals = normals_of(&s.positions, &s.tris);
        let first_vertex = t7.vertices.len() as u32;
        let base_index = t7.indices.len() as u32;
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for (i, p) in s.positions.iter().enumerate() {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
            let n = normals[i];
            t7.vertices.push(WorldVertex {
                xyz: *p,
                color: 0xFFFF_FFFF,
                uv: s.uvs.get(i).copied().unwrap_or_default(),
                normal: n,
                tangent: tangent_of(n),
            });
        }
        t7.indices.extend(s.tris.iter().flatten());
        t7.surfaces.push(WorldSurface {
            first_vertex,
            vertex_count: s.positions.len() as u32,
            tri_count: s.tris.len() as u16,
            base_index,
            material: s.material,
            flags: 0,
            mins: lo,
            maxs: hi,
        });
    }
    // The bounds of what is drawn.
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for s in &t7.surfaces {
        for k in 0..3 {
            lo[k] = lo[k].min(s.mins[k]);
            hi[k] = hi[k].max(s.maxs[k]);
        }
    }
    (t7.mins, t7.maxs) = if t7.surfaces.is_empty() {
        (map.mins, map.maxs)
    } else {
        (lo, hi)
    };
    report.push(format!(
        "jup world drawn: {} surfaces ({sheets} sheets wider than {SHEET} left out), bounds {:?} {:?}",
        t7.surfaces.len(),
        t7.mins,
        t7.maxs
    ));
    let (ground, area) = floor_points(map);
    // Collision only where the map is played: not the backdrop beyond the
    // play area (a big map's far tiles).
    if let Some(area) = area {
        t7.no_collision = t7
            .surfaces
            .iter()
            .map(|s| !area.reaches(s.mins, s.maxs))
            .collect();
        let left = t7.no_collision.iter().filter(|&&n| n).count();
        report.push(format!(
            "jup collision: play area {:.0} around ({:.0}, {:.0}); {left} of {} surfaces drawn only",
            area.radius,
            area.mid[0],
            area.mid[1],
            t7.surfaces.len()
        ));
    }
    let mut floor = ents::path_nodes(&map.entities);
    let source = if floor.is_empty() {
        floor = ground;
        "the world's floor"
    } else {
        "the path nodes"
    };
    let spawns = ents::spawn_entities(&floor);
    report.push(format!(
        "jup spawns: made over {source} ({} points)",
        floor.len()
    ));
    t7.entities = if map.entities.trim().is_empty() {
        format!("{{\n\"classname\" \"worldspawn\"\n}}\n{spawns}")
    } else {
        format!("{}{spawns}", map.entities)
    };
    t7
}

/// The largest a picture is drawn at.
const PICTURE_MAX: u32 = 1024;

/// Does a BC1 picture cut holes (a tenth of its blocks in the
/// punch-through mode)? Its material is then drawn alpha-tested.
fn bc1_cuts(dxgi: u32, data: &[u8]) -> bool {
    if dxgi != 71 || data.len() < 8 {
        return false;
    }
    let blocks = data.chunks_exact(8);
    let n = blocks.len();
    let holes = blocks
        .filter(|b| u16::from_le_bytes([b[0], b[1]]) <= u16::from_le_bytes([b[2], b[3]]))
        .count();
    holes * 10 >= n
}

/// The materials as the Black Ops III builder reads them, each with its
/// colour picture's pixels (fetched from the install's packages, a few
/// threads at once). Materials keep the map's order, so a surface's
/// material index stays.
fn materials_into(
    map: &JupMap,
    t7: &mut T7Map,
    install: &Path,
    oodle: &dyn fastfile_jup::oodle::Decompressor,
    report: &mut Vec<String>,
) {
    if map.materials.is_empty() {
        return;
    }
    let started = std::time::Instant::now();
    let paks = Packages::open(install);
    let wanted: Vec<usize> = map.materials.iter().filter_map(|m| m.colour).collect();
    let mut wanted_sorted = wanted.clone();
    wanted_sorted.sort_unstable();
    wanted_sorted.dedup();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: std::sync::Mutex<
        Vec<(usize, Result<fastfile_jup::map::materials::Level, String>)>,
    > = std::sync::Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().clamp(2, 8));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&img) = wanted_sorted.get(k) else {
                        break;
                    };
                    let level = paks.level(&map.images[img], PICTURE_MAX, oodle);
                    if let Ok(mut r) = results.lock() {
                        r.push((img, level));
                    }
                }
            });
        }
    });
    let mut decoded = std::collections::HashMap::new();
    let mut failures = Vec::new();
    for (img, level) in results.into_inner().unwrap_or_default() {
        match level {
            Ok(l) => {
                decoded.insert(img, l);
            }
            Err(e) => failures.push(e),
        }
    }
    let mut cut = 0usize;
    for m in &map.materials {
        let mut mat = MapMaterial {
            name: format!("jup_{:016x}", m.hash),
            technique_set: String::new(),
            textures: Vec::new(),
        };
        if let Some(c) = m.colour
            && let Some(l) = decoded.get(&c)
        {
            if bc1_cuts(l.dxgi, &l.data) {
                mat.name.push_str("_alphatest");
                cut += 1;
            }
            let name = format!("jup_img_{:016x}", map.images[c].hash);
            t7.add_image(
                name.clone(),
                ImageLevels {
                    format: l.dxgi,
                    width: l.width,
                    height: l.height,
                    levels: 1,
                    data: l.data.clone(),
                    part: None,
                },
            );
            mat.textures.push((0, name));
        }
        t7.materials.push(mat);
    }
    report.push(format!(
        "jup pictures: {} of {} colour pictures from {} packages in {:.1}s ({cut} materials alpha-tested){}",
        decoded.len(),
        wanted_sorted.len(),
        paks.len(),
        started.elapsed().as_secs_f32(),
        failures
            .first()
            .map(|e| format!("; first missing: {e}"))
            .unwrap_or_default()
    ));
}

/// Loads a Modern Warfare III map zone into the match's world. A zone that
/// cannot be read becomes a gap with the reason, never a crash.
pub(crate) fn load_jup_world(
    path: &Path,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
) -> LoadedWorld {
    let src = MapSources::beside(path);
    let mut report = vec![format!(
        "game: JUP / Modern Warfare III ({})",
        path.display()
    )];
    let stage = progress.begin_scoped(StageId::MapAssets, "modern warfare iii zone", None);
    let started = std::time::Instant::now();
    let read = std::panic::catch_unwind(|| {
        let oodle = fastfile_jup::oodle::load(&src.install).map_err(|e| e.to_string())?;
        JupMap::load(&src, oodle.as_ref()).map(|m| (m, oodle))
    });
    let (map, oodle) = match read {
        Ok(Ok(read)) => {
            stage.done();
            read
        }
        Ok(Err(error)) => {
            stage.fail();
            diag::warn!(World, "bo2mp jup: {}: {error}", path.display());
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Modern Warfare III zone: {error}"),
                Some("assets::lane::jup_map::load_world/zone"),
            );
        }
        Err(panic) => {
            stage.fail();
            let what = panic
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown error".to_owned());
            diag::warn!(
                World,
                "bo2mp jup: {}: reader panicked: {what}",
                path.display()
            );
            return LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                format!("Modern Warfare III zone could not be read: {what}"),
                Some("assets::lane::jup_map::load_world/panic"),
            );
        }
    };
    report.push(format!(
        "jup zone: read in {:.1}s",
        started.elapsed().as_secs_f32()
    ));
    report.extend(map.report.iter().map(|l| format!("jup {l}")));
    let mut t7 = as_t7(&map, &mut report);
    let stage = progress.begin_scoped(StageId::MapAssets, "modern warfare iii pictures", None);
    materials_into(&map, &mut t7, &src.install, oodle.as_ref(), &mut report);
    stage.done();
    let loaded = super::t7_map::build_t7_world(path, &t7, report, progress, material_seed);
    diag::info!(
        World,
        "bo2mp jup: {} loaded: {}",
        path.display(),
        loaded
            .report
            .iter()
            .filter(|l| l.starts_with("jup ")
                || l.starts_with("world mesh")
                || l.starts_with("t7 "))
            .cloned()
            .collect::<Vec<_>>()
            .join("; ")
    );
    loaded
}
