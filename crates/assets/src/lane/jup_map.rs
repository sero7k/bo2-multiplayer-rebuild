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
use fastfile_jup::map::{JupMap, MapSources, ents};
use fastfile_t7::map::{T7Map, WorldSurface, WorldVertex};

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

/// Floor points when the map has no path nodes: the middles of the
/// world's upward triangles of some size, in the box most of them are in.
fn floor_points(map: &JupMap) -> Vec<[f32; 3]> {
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
            if area2 > 2.0 * 32.0 * 32.0 && f[2] / area2 > 0.85 {
                pts.push(std::array::from_fn(|k| (a[k] + b[k] + c[k]) / 3.0));
            }
        }
    }
    // The middle 80% on each axis (sky boxes and far vistas out).
    for k in 0..2 {
        let mut v: Vec<f32> = pts.iter().map(|p: &[f32; 3]| p[k]).collect();
        v.sort_by(f32::total_cmp);
        if v.len() > 10 {
            let (lo, hi) = (v[v.len() / 10], v[v.len() * 9 / 10]);
            pts.retain(|p| p[k] >= lo && p[k] <= hi);
        }
    }
    pts
}

/// The map as a Black Ops III map read.
fn as_t7(map: &JupMap, report: &mut Vec<String>) -> T7Map {
    let mut t7 = T7Map::default();
    t7.name = map.name.clone();
    (t7.mins, t7.maxs) = (map.mins, map.maxs);
    for s in &map.surfaces {
        if s.tris.is_empty()
            || s.positions.is_empty()
            || s.positions.len() > usize::from(u16::MAX)
            || s.tris.len() > usize::from(u16::MAX)
        {
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
            material: None,
            flags: 0,
            mins: lo,
            maxs: hi,
        });
    }
    let mut floor = ents::path_nodes(&map.entities);
    let source = if floor.is_empty() {
        floor = floor_points(map);
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
        JupMap::load(&src, oodle.as_ref())
    });
    let map = match read {
        Ok(Ok(map)) => {
            stage.done();
            map
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
    let t7 = as_t7(&map, &mut report);
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
