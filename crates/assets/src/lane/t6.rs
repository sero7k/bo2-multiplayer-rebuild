//! bo2zm: the Black Ops II lane.
//!
//! A T6 map loads through `asset_t6`: the zone is walked once, capturing the
//! world, collision and map entities; `asset_world` turns them into the
//! engine's world draw and clip collision. Materials have no route yet, so
//! world surfaces draw through the renderer's diagnostic path; the player
//! starts at the map's own `info_player_start` through the start scripts in
//! `t6_scripts`.

use std::collections::BTreeMap;
use std::path::Path;

use super::{CommonCensus, LoadedWorld, MaterialPopulation, ZoneLane};
use crate::lane_capability::{LaneStatus, PreparedCapability};
use crate::session_load::PreparedWorld;
use asset_core::ZoneGame;
use asset_transport::progress::{LoadProgress, StageId};
use asset_transport::{T6ZoneMemory, ZoneImage};
use asset_world::WorldDrawPolicy;

pub struct T6Lane;

impl T6Lane {
    pub const GAME: ZoneGame = ZoneGame::T6;
    pub const CAPABILITIES: &'static [(PreparedCapability, LaneStatus)] = &[
        (PreparedCapability::Envelope, LaneStatus::SupportedPopulated),
        (
            PreparedCapability::PreparedWorld,
            LaneStatus::SupportedPopulated,
        ),
        (
            PreparedCapability::CollisionSpawns,
            LaneStatus::SupportedPopulated,
        ),
        (
            PreparedCapability::WeaponCatalog,
            LaneStatus::MissingDecoder,
        ),
        (PreparedCapability::BodySkeleton, LaneStatus::MissingDecoder),
        (
            PreparedCapability::PlayableFfa,
            LaneStatus::UnsupportedByRuntimeProfile,
        ),
    ];
}

/// One line per zone: asset count by type, or why the asset list did not open.
pub fn asset_census(path: &Path, image: &ZoneImage) -> String {
    let zone = path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let header = match image.t6_header() {
        Ok(header) => header,
        Err(error) => return format!("t6 census {zone}: zone header: {error}"),
    };
    let mut memory = T6ZoneMemory::for_header(&header);
    let mut stream = match memory.stream(&image.bytes) {
        Ok(stream) => stream,
        Err(error) => return format!("t6 census {zone}: zone arenas: {error}"),
    };
    let table = match fastfile_t6::open_asset_table(&mut stream) {
        Ok(table) => table,
        Err(error) => return format!("t6 census {zone}: asset list: {error}"),
    };
    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for i in 0..table.count() {
        match table.raw_kind(&stream, i) {
            Ok(raw) => *counts.entry(raw).or_default() += 1,
            Err(error) => return format!("t6 census {zone}: asset {i}: {error}"),
        }
    }
    let by_type: Vec<String> = counts
        .iter()
        .map(|(raw, n)| match fastfile_t6::AssetType::from_u32(*raw) {
            Some(ty) => format!("{}={n}", ty.name()),
            None => format!("pool{raw:#x}={n}"),
        })
        .collect();
    format!(
        "t6 census {zone}: {} assets, {} script strings: {}",
        table.count(),
        table.strings.count(),
        by_type.join(" ")
    )
}

/// bo2zm fix list 2: the clip map's placed models (`staticModelList`: cars,
/// fences, trees, porches, rocks) with their models' collision surfaces, so
/// bullets and thrown grenades hit them as in BO2 (player movement keeps to
/// the clip brushes around them). Returns (placed, with triangles).
fn attach_t6_static_models(
    capture: &asset_t6::ZoneCapture,
    clip: &mut asset_world::ClipCollision,
) -> (usize, usize) {
    let Some(source) = capture.clip.as_ref() else {
        return (0, 0);
    };
    let mid_half = |lo: [f32; 3], hi: [f32; 3]| {
        (
            [0, 1, 2].map(|a| (lo[a] + hi[a]) * 0.5),
            [0, 1, 2].map(|a| ((hi[a] - lo[a]) * 0.5).max(0.0)),
        )
    };
    let mut with_tris = 0;
    for (index, sm) in source.static_models.iter().enumerate() {
        let Some(model) = sm.model.and_then(|k| capture.xmodels.get(k.index)) else {
            continue;
        };
        let surfs: Vec<clipmap_iw4::XModelCollSurf> = model
            .coll_surfs
            .iter()
            .map(|s| {
                let (midpoint, half_size) = mid_half(s.mins, s.maxs);
                clipmap_iw4::XModelCollSurf {
                    tris: s
                        .tris
                        .iter()
                        .map(|t| clipmap_iw4::XModelCollTri {
                            plane: t[0],
                            svec: t[1],
                            tvec: t[2],
                        })
                        .collect(),
                    midpoint,
                    half_size,
                    bone_idx: s.bone_idx,
                    contents: s.contents,
                    surf_flags: s.surf_flags,
                }
            })
            .collect();
        if model.coll_lod >= 0 && surfs.iter().any(|s| !s.tris.is_empty()) {
            with_tris += 1;
        }
        let (bounds_mid, bounds_half) = mid_half(sm.absmin, sm.absmax);
        clip.static_models.push(asset_world::ClipPlacedStaticModel {
            index: index as u32,
            name: model.name.clone(),
            model: clipmap_iw4::ClipStaticModel {
                origin: sm.origin,
                inv_scaled_axis: sm.inv_scaled_axis,
                bounds_mid,
                bounds_half,
                coll: clipmap_iw4::XModelColl {
                    coll_lod: model.coll_lod,
                    // The placement's own contents (a placement can opt out).
                    contents: sm.contents,
                    surfs,
                },
            },
        });
    }
    (clip.static_models.len(), with_tris)
}

/// The map's player start. Zombies maps start players at `initial_spawn`
/// structs whose `script_string` names the mode and location
/// (`zstandard_nuked`); their `info_player_start` can sit inside player
/// clip. Then any `initial_spawn` struct; a multiplayer map's own first
/// free-for-all spawn point (bo2mp: BO2 never starts a player at the
/// `info_player_start`, which Hijacked lacks and Overflow puts in a
/// dumpster); the `info_player_start`; and last the world centre.
fn player_start(
    capture: &asset_t6::ZoneCapture,
    map: &str,
    report: &mut Vec<String>,
) -> ([f32; 3], f32) {
    let location = map.strip_prefix("zm_").unwrap_or(map);
    // A chosen start (Tranzit's Town survival: `zstandard_town`) first.
    let chosen = frame::bo2_zm_start(map).map(|(loc, mode)| format!("{mode}_{loc}"));
    let wanted: Vec<String> = chosen
        .into_iter()
        .chain([
            format!("zstandard_{location}"),
            format!("zclassic_{location}"),
        ])
        .collect();
    let ents: Vec<asset_t6::MapEntity> = capture
        .map_ents
        .iter()
        .flat_map(|text| asset_t6::parse_entities(text))
        .collect();
    let initial = || {
        ents.iter()
            .filter(|e| e.get("script_noteworthy") == Some("initial_spawn"))
    };
    let named = wanted.iter().find_map(|w| {
        initial().find(|e| {
            e.get("script_string")
                .is_some_and(|s| s.split_whitespace().any(|t| t == w))
        })
    });
    // bo2mp: a multiplayer map's spawn points by kind (`mp_dm_spawn`,
    // `mp_tdm_spawn`, `mp_tdm_spawn_allies_start`, ...).
    let is_mp_spawn = |e: &&asset_t6::MapEntity| {
        let c = e.classname();
        c.starts_with("mp_") && c.contains("_spawn")
    };
    let mp = super::t6_m2::is_mp_map(map);
    if mp {
        let mut kinds: std::collections::BTreeMap<&str, usize> = Default::default();
        for e in ents.iter().filter(is_mp_spawn) {
            *kinds.entry(e.classname()).or_default() += 1;
        }
        report.push(format!(
            "t6 mp spawns: {}",
            kinds
                .iter()
                .map(|(k, n)| format!("{k} {n}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let picks = [
        named.map(|e| (e, "initial_spawn for this location")),
        initial().next().map(|e| (e, "initial_spawn")),
        ents.iter()
            .find(|e| e.classname() == "mp_dm_spawn")
            .or_else(|| ents.iter().find(is_mp_spawn))
            .filter(|_| mp)
            .map(|e| (e, e.classname())),
        ents.iter()
            .find(|e| e.classname() == "info_player_start")
            .map(|e| (e, "info_player_start")),
    ];
    for (e, what) in picks.into_iter().flatten() {
        if let Some(origin) = e.vec3("origin") {
            let yaw = e.vec3("angles").map_or(0.0, |a| a[1]);
            report.push(format!(
                "t6 spawn: {what} at [{:.1}, {:.1}, {:.1}] yaw {yaw:.1}",
                origin[0], origin[1], origin[2]
            ));
            return (origin, yaw);
        }
    }
    let w = capture.world.as_ref();
    let c = w.map_or([0.0; 3], |w| {
        [
            (w.mins[0] + w.maxs[0]) * 0.5,
            (w.mins[1] + w.maxs[1]) * 0.5,
            w.maxs[2],
        ]
    });
    report.push("t6 spawn gap: no info_player_start; world centre used".to_owned());
    (c, 0.0)
}

/// bo2zm: the props a map zone only names. TranZit's placed models live
/// in its area zones (`zm_transit_gump_busstation.ff`, ...), which the game
/// streams in by area; the map zone holds a comma-named reference to each
/// (the Bus Depot's route-map kiosk, most of the Diner, Farm and Town).
#[derive(Default)]
struct AreaModels {
    captures: Vec<asset_t6::ZoneCapture>,
    /// A referenced model's name -> (capture, its model index).
    models: std::collections::HashMap<String, (usize, usize)>,
    /// Per capture: its material index -> catalog row.
    local: Vec<std::collections::HashMap<usize, usize>>,
    /// The map zone's own materials by name (an area material can be a
    /// reference to one of them).
    map_materials: std::collections::HashMap<String, usize>,
}

impl AreaModels {
    /// The area zones beside the map that define the models its static
    /// models reference; none for a map whose props are all its own.
    fn read(path: &Path, map: &str, capture: &asset_t6::ZoneCapture, world: &asset_t6::WorldRef) -> Self {
        let wanted: std::collections::BTreeSet<&str> = world
            .static_models
            .iter()
            .filter_map(|sm| sm.model)
            .filter_map(|k| capture.xmodels.get(k.index))
            .filter_map(|m| m.name.strip_prefix(','))
            .collect();
        if wanted.is_empty() {
            return Self::default();
        }
        let dir = asset_transport::t6_extra::install_zone_dir(path);
        let prefix = format!("{map}_gump_");
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".ff"))
            })
            .collect();
        files.sort();
        let mut out = Self::default();
        for file in files {
            let Ok(area) = asset_t6::capture_zone(&file) else { continue };
            let ci = out.captures.len();
            for (mi, m) in area.xmodels.iter().enumerate() {
                if wanted.contains(m.name.as_str()) {
                    out.models.entry(m.name.clone()).or_insert((ci, mi));
                }
            }
            out.captures.push(area);
        }
        out.map_materials = capture
            .materials
            .iter()
            .enumerate()
            .filter(|(_, m)| !m.name.is_empty() && !m.name.starts_with(','))
            .map(|(i, m)| (m.name.clone(), i))
            .collect();
        out
    }

    /// Each area model surface's material: (capture, material index), the
    /// area's own or, for a reference, the map zone's (capture `None`).
    fn surface_materials(&self) -> impl Iterator<Item = (Option<usize>, usize)> + '_ {
        self.models.values().flat_map(move |&(ci, mi)| {
            let c = &self.captures[ci];
            c.xmodels[mi].lod0.iter().filter_map(move |srf| {
                let k = srf.material?;
                let name = &c.materials.get(k.index)?.name;
                match name.strip_prefix(',') {
                    Some(n) => self.map_materials.get(n).map(|&i| (None, i)),
                    None => Some((Some(ci), k.index)),
                }
            })
        })
    }

    /// Link every area model's own materials into the catalog, images by
    /// name from the map zone and the areas.
    fn link(
        &mut self,
        materials: &mut asset_material::MaterialCatalog,
        capture: &asset_t6::ZoneCapture,
        packs: Option<&asset_t6::PackSet>,
        report: &mut Vec<String>,
    ) {
        let mut images: std::collections::HashMap<String, asset_t6::ImageRef> = Default::default();
        for img in capture.images.iter().chain(self.captures.iter().flat_map(|c| c.images.iter())) {
            if !img.name.is_empty() && !img.name.starts_with(',') {
                images.entry(img.name.clone()).or_insert_with(|| img.clone());
            }
        }
        let mut wanted: Vec<std::collections::BTreeSet<usize>> = vec![Default::default(); self.captures.len()];
        for (ci, mi) in self.surface_materials() {
            if let Some(ci) = ci {
                wanted[ci].insert(mi);
            }
        }
        self.local = self
            .captures
            .iter()
            .zip(&wanted)
            .map(|(c, want)| {
                let linked = super::t6_materials::link_materials_with(
                    materials,
                    c,
                    packs,
                    want.iter().copied(),
                    &c.zone,
                    Some(&images),
                );
                report.extend(linked.report.into_iter().map(|l| format!("t6 area ({}): {l}", c.zone)));
                linked.local
            })
            .collect();
        report.push(format!(
            "t6 area zones: {} read, {} referenced props found",
            self.captures.len(),
            self.models.len()
        ));
    }

    /// The real model for a map model, with its surfaces' catalog rows.
    fn model<'a>(
        &'a self,
        own: &'a asset_t6::XModelRef,
        map_local: &'a std::collections::HashMap<usize, usize>,
    ) -> (&'a asset_t6::XModelRef, Box<dyn Fn(asset_t6::AssetKey) -> Option<usize> + 'a>) {
        let Some(&(ci, mi)) = own.name.strip_prefix(',').and_then(|n| self.models.get(n)) else {
            return (own, Box::new(move |k| map_local.get(&k.index).copied()));
        };
        let c = &self.captures[ci];
        let local = self.local.get(ci);
        (
            &c.xmodels[mi],
            Box::new(move |k| {
                let name = &c.materials.get(k.index)?.name;
                match name.strip_prefix(',') {
                    Some(n) => map_local.get(self.map_materials.get(n)?).copied(),
                    None => local?.get(&k.index).copied(),
                }
            }),
        )
    }
}

/// The placed static models: one mesh per model used (its first LOD), a
/// mesh of its own for each instance with baked vertex light, and every
/// placement with its light. Instances without baked light take the light
/// grid at their origin, scaled by the ratio measured on the baked ones
/// (the grid's coefficient scale is not stored with it).
fn build_props(
    capture: &asset_t6::ZoneCapture,
    world: &asset_t6::WorldRef,
    local_materials: &std::collections::HashMap<usize, usize>,
    areas: &AreaModels,
    clip: Option<&asset_world::ClipCollision>,
    report: &mut Vec<String>,
) -> (
    Vec<asset_world::ModelMesh>,
    Vec<Option<asset_world::StaticModelPlacement>>,
) {
    use bevy::prelude::{Mat3, Quat, Transform, Vec3};

    let grid = world
        .light_grid
        .as_ref()
        .and_then(asset_world::t6_light_grid);
    // Places outside the light grid (the distant terrain) take their light
    // from a sky grid volume: one coefficient set, one primary light and
    // its visibility.
    let raw_grid = world.light_grid.as_ref();
    let in_grid = |p: [f32; 3]| -> bool {
        let Some(g) = raw_grid else { return false };
        let cell = [
            (p[0].floor() as i32).wrapping_add(0x20000) >> 5,
            (p[1].floor() as i32).wrapping_add(0x20000) >> 5,
            (p[2].floor() as i32).wrapping_add(0x20000) >> 6,
        ];
        (0..3).all(|a| cell[a] >= i32::from(g.mins[a]) && cell[a] <= i32::from(g.maxs[a]))
    };
    let sky_volume = |p: [f32; 3]| {
        raw_grid.and_then(|g| {
            g.sky_volumes
                .iter()
                .find(|v| (0..3).all(|a| p[a] >= v.mins[a] && p[a] <= v.maxs[a]))
        })
    };
    let sky_light = |v: &asset_t6::SkyGridVolumeRef| -> Option<[f32; 3]> {
        let g = raw_grid?;
        let at = usize::from(v.colors_index) * 54;
        let set = g.coeffs.get(at..at + 6)?;
        Some([0usize, 1, 2].map(|c| {
            let raw = u16::from_le_bytes([set[c * 2], set[c * 2 + 1]]);
            (f32::from(raw) - 32768.0) / 32768.0
        }))
    };
    let mut sky_n = 0usize;
    let mut own_ratios: Vec<f32> = Vec::new();
    let mut meshes: Vec<asset_world::ModelMesh> = Vec::new();
    let mut shared: std::collections::HashMap<usize, Option<usize>> = Default::default();
    let mut ratios: Vec<f32> = Vec::new();
    let mut ratio_grid: Vec<(f32, f32)> = Vec::new();
    let mut pending: Vec<(usize, usize, Option<[f32; 3]>, bool)> = Vec::new();
    for (i, sm) in world.static_models.iter().enumerate() {
        let Some(key) = sm.model else { continue };
        // A referenced model (TranZit's) draws from its area zone.
        let (xm, material) = areas.model(&capture.xmodels[key.index], local_materials);
        let base = *shared.entry(key.index).or_insert_with(|| {
            let mesh = asset_world::build_t6_model_mesh(xm, |k| material(k))?;
            meshes.push(mesh);
            Some(meshes.len() - 1)
        });
        let Some(base) = base else { continue };
        // bo2zm M3 fix list 3: also the light the map's compiler stored for
        // this placement (its own light grid colour set, `colorsIndex`). A
        // prop without baked vertex light was lit only from the grid at its
        // origin, on the ground; grid points buried in a prop or a mound are
        // near black, and the bunker behind the teal house (its base against
        // a dirt mound) drew nearly black: his "extra dark ... bad shading".
        let xm = &capture.xmodels[key.index];
        let mid: [f32; 3] = std::array::from_fn(|i| (xm.mins[i] + xm.maxs[i]) * 0.5 * sm.scale);
        let light_at: [f32; 3] = std::array::from_fn(|i| {
            sm.origin[i] + mid[0] * sm.axis[0][i] + mid[1] * sm.axis[1][i] + mid[2] * sm.axis[2][i]
        });
        let own = raw_grid.and_then(|g| {
            let at = usize::from(sm.colors_index) * 54;
            let set = g.coeffs.get(at..at + 6)?;
            Some([0usize, 1, 2].map(|c| {
                let raw = u16::from_le_bytes([set[c * 2], set[c * 2 + 1]]);
                (f32::from(raw) - 32768.0) / 32768.0
            }))
        });
        let outside = !in_grid(light_at);
        let sampled = if outside {
            sky_volume(light_at).and_then(sky_light)
        } else {
            grid.as_ref()
                .and_then(|g| asset_world::t6_grid_light(g, light_at))
        };
        // The grid as it was read before (at the origin): what the brightness
        // scale is measured against, so the props he has seen keep their
        // level and only the ones lit from under the ground change.
        let old_sample = if in_grid(sm.origin) {
            grid.as_ref().and_then(|g| asset_world::t6_grid_light(g, sm.origin))
        } else {
            sky_volume(sm.origin).and_then(sky_light)
        };
        if let (Some(o), Some(g)) = (own, old_sample) {
            let (lo, lg) = ((o[0] + o[1] + o[2]) / 3.0, (g[0] + g[1] + g[2]) / 3.0);
            if lg > 1e-4 && sm.lmap_colors.is_empty() {
                own_ratios.push(lo / lg);
                if std::env::var_os("IW4L_T6_SMLIGHT_ALL").is_some() {
                    diag::info!(
                        World,
                        "bo2zm t6 smlight all {:.2} {} at ({:.0} {:.0} {:.0}) own {lo:.3} grid {lg:.3} set {}",
                        lo / lg,
                        xm.name,
                        sm.origin[0],
                        sm.origin[1],
                        sm.origin[2],
                        sm.colors_index
                    );
                }
            }
        }
        // Either can come from a point buried in something (the compiler's
        // set for small rubble on the ground is near black, the grid under
        // the bunker too): the brighter of the two.
        let luma = |l: [f32; 3]| (l[0] + l[1] + l[2]) / 3.0;
        let grid_light = match (own, old_sample.or(sampled)) {
            (Some(o), Some(g)) => Some(if luma(o) > luma(g) { o } else { g }),
            (o, g) => o.or(g),
        };
        // IW4L_T6_SMLIGHT_LOG=<name part>: the light at the base and at the
        // middle for such placements (test aid).
        if let Ok(want) = std::env::var("IW4L_T6_SMLIGHT_LOG")
            && xm.name.contains(&want)
        {
            let at_base = grid.as_ref().and_then(|g| asset_world::t6_grid_light(g, sm.origin));
            diag::info!(
                World,
                "bo2zm t6 smlight {} at {:?}: grid at base {:?}, used {:?} (own set {})",
                xm.name,
                sm.origin,
                at_base,
                grid_light,
                sm.colors_index
            );
        }
        let baked = (!sm.lmap_colors.is_empty())
            .then(|| asset_world::t6_vertex_lit_mesh(&meshes[base], &sm.lmap_colors))
            .flatten();
        let (mesh, vertex_lit) = match baked {
            Some(lit) => {
                // Measure: the baked light's mean against the grid's.
                if let Some(g) = old_sample {
                    let mean: f32 = sm
                        .lmap_colors
                        .iter()
                        .map(|c| {
                            let b = c.to_le_bytes();
                            (0..3)
                                .map(|k| (f32::from(b[k]) / 255.0).powi(2) * 32.0)
                                .sum::<f32>()
                                / 3.0
                        })
                        .sum::<f32>()
                        / sm.lmap_colors.len() as f32;
                    let grid_mean = (g[0] + g[1] + g[2]) / 3.0;
                    if grid_mean > 1e-4 {
                        ratios.push(mean / grid_mean);
                        ratio_grid.push((grid_mean, mean / grid_mean));
                    }
                }
                meshes.push(lit);
                (meshes.len() - 1, true)
            }
            None => (base, false),
        };
        pending.push((i, mesh, grid_light, vertex_lit));
    }
    ratios.sort_by(f32::total_cmp);
    // A map with (almost) no baked props has nothing to measure on: Tranzit
    // has none, and 1.0 drew its 5,059 props ~13x too dark. The ratio is
    // the grid's storage scale, 13.1-13.4 on Nuketown, Buried and Nacht.
    const TYPICAL_GRID_SCALE: f32 = 13.354;
    let scale = if ratios.len() >= 20 {
        ratios[ratios.len() / 2]
    } else {
        TYPICAL_GRID_SCALE
    };
    let at = |q: f32| {
        ratios
            .get(((ratios.len() as f32 - 1.0) * q).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    // The same over the props where the grid is brightest (half of them):
    // a dark grid sample (an origin inside a wall) makes a ratio of noise.
    ratio_grid.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut bright: Vec<f32> = ratio_grid
        .iter()
        .take(ratio_grid.len().div_ceil(2))
        .map(|p| p.1)
        .collect();
    bright.sort_by(f32::total_cmp);
    let bright_median = bright.get(bright.len() / 2).copied().unwrap_or(1.0);
    // bo2mp: when the dark samples carry the median (more than five times
    // the brighter half's: Cargo 773 against 38, its grid-lit props drawn
    // white), the brighter half's is the scale. Every other map measured
    // keeps its median (Nuketown 18 against 15, Raid 23 against 5).
    let scale = if ratios.len() >= 20 && scale > bright_median * 5.0 {
        bright_median
    } else {
        scale
    };
    report.push(format!(
        "t6 props: baked light against the grid, {} props: 10% {:.2}, 25% {:.2}, median {:.2}, 75% {:.2}, 90% {:.2}; brighter half median {bright_median:.2}; scale {scale:.2}",
        ratios.len(),
        at(0.1),
        at(0.25),
        at(0.5),
        at(0.75),
        at(0.9)
    ));
    own_ratios.sort_by(f32::total_cmp);
    let pct = |q: f32| {
        own_ratios
            .get(((own_ratios.len() as f32 - 1.0) * q).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    report.push(format!(
        "t6 props: own light against the grid at the origin, {} props: 10% {:.2}, median {:.2}, 90% {:.2}; {} over 3x",
        own_ratios.len(),
        pct(0.1),
        pct(0.5),
        pct(0.9),
        own_ratios.iter().filter(|r| **r > 3.0).count()
    ));
    let mut placements = vec![None; world.static_models.len()];
    let (mut gridded, mut baked_n, mut unlit) = (0usize, 0usize, 0usize);
    let mut visible_n = 0usize;
    for (i, mesh, grid_light, vertex_lit) in pending {
        let sm = &world.static_models[i];
        let packed_lighting = if vertex_lit {
            baked_n += 1;
            Some([0, 0, 0, 255])
        } else if let Some(light) = grid_light {
            gridded += 1;
            let encode = |v: f32| ((v * scale / 32.0).clamp(0.0, 1.0).sqrt() * 255.0).round() as u8;
            Some([encode(light[0]), encode(light[1]), encode(light[2]), 0])
        } else {
            unlit += 1;
            None
        };
        // The primary light the game lights this placement with, and how
        // much of it reaches it (the light grid's visibility there).
        let primary = sm.primary_light_index;
        let visibility = if !in_grid(sm.origin) {
            sky_n += 1;
            sky_volume(sm.origin)
                .filter(|v| v.primary_light_index == primary)
                .map_or(0, |v| v.visibility)
        } else {
            grid.as_ref()
                .and_then(|g| asset_world::t6_grid_visibility(g, sm.origin, primary))
                .unwrap_or(0)
        };
        if visibility > 0 {
            visible_n += 1;
        }
        let basis = Mat3::from_cols(
            Vec3::from_array(sm.axis[0]),
            Vec3::from_array(sm.axis[1]),
            Vec3::from_array(sm.axis[2]),
        );
        placements[i] = Some(asset_world::StaticModelPlacement {
            mesh,
            transform: Transform {
                translation: Vec3::from_array(sm.origin),
                rotation: Quat::from_mat3(&basis),
                scale: Vec3::splat(sm.scale),
            },
            origin: sm.origin,
            axis: sm.axis,
            scale: sm.scale,
            cull_dist: sm.cull_dist.clamp(0.0, f32::from(u16::MAX)) as u16,
            reflection_probe_index: sm.reflection_probe_index,
            primary_light_index: sm.primary_light_index,
            flags: sm.flags as u8,
            packed_lighting,
            t6_primary: [primary, visibility],
        });
    }
    // bo2zm M2: what lights moving things (the first-person gun, effects).
    asset_world::install_t6_light_sampler(raw_grid.map(|g| asset_world::T6LightSampler {
        grid: grid.clone(),
        mins: g.mins,
        maxs: g.maxs,
        sky: g
            .sky_volumes
            .iter()
            .map(|v| {
                (
                    v.mins,
                    v.maxs,
                    sky_light(v).unwrap_or_default(),
                    v.primary_light_index,
                    v.visibility,
                )
            })
            .collect(),
        scale,
        sun_primary: g.sun_primary_light_index as u8,
        clip: clip.map(|c| std::sync::Arc::new(c.clone())),
        cache: Default::default(),
        foreign: None,
    }));
    report.push(format!(
        "t6 props: {} placed, {} meshes; {baked_n} with baked vertex light, {gridded} lit by the light grid (scale {scale:.3} measured on {} baked), {unlit} without light; {visible_n} see their primary light; {sky_n} outside the grid (sky volume)",
        placements.iter().filter(|p| p.is_some()).count(),
        meshes.len(),
        ratios.len()
    ));
    (meshes, placements)
}

/// The map's fog as its createart script sets it, from the map's zone or
/// its patch zone (`<map>_patch.ff` beside it).
fn art_fog(capture: &asset_t6::ZoneCapture, path: &Path, map: &str) -> Option<asset_t6::ArtFog> {
    let want = format!("script:maps/mp/createart/{map}_art.gsc");
    let find = |c: &asset_t6::ZoneCapture| {
        c.raw_files
            .iter()
            .find(|(name, _)| *name == want)
            .and_then(|(_, bytes)| asset_t6::parse_art_fog(bytes))
    };
    find(capture).or_else(|| {
        let patch = asset_transport::t6_extra::file_in(
            &asset_transport::t6_extra::install_zone_dir(path),
            &format!("{map}_patch.ff"),
        );
        asset_t6::capture_zone(&patch).ok().as_ref().and_then(find)
    })
}

/// bo2mp: the map's final colour: its vision file (`vision/<map>.vision`)
/// and band 0 of its colour table (`GfxWorld::lutMaterial`'s image), built
/// into the table BO2's `hdr_create_lut2dv` makes (`asset_world::t6_grade`).
pub(crate) fn t6_grade(
    capture: &asset_t6::ZoneCapture,
    world: &asset_t6::WorldRef,
    packs: Option<&asset_t6::PackSet>,
    map: &str,
) -> (Option<asset_world::T6Grade>, String) {
    let want = format!("vision/{map}.vision");
    let vision = capture
        .raw_files
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&want))
        .and_then(|(_, bytes)| asset_world::parse_t6_vision(&String::from_utf8_lossy(bytes)));
    let image = world
        .lut_material
        .and_then(|k| capture.materials.get(k.index))
        .and_then(|m| m.textures.first())
        .and_then(|t| t.image)
        .and_then(|k| capture.images.get(k.index));
    let band = 1024 * 32 * 4;
    let base = match (image, packs) {
        (Some(img), Some(packs)) => match packs.locate(img) {
            Some(asset_t6::ImageSource::Pack(i, entry)) => {
                packs.packs[i].read(entry).ok().and_then(|bytes| {
                    let iwi = ipak_t6::parse_iwi(&bytes).ok()?;
                    (iwi.format == ipak_t6::IwiFormat::Rgba8 && iwi.width == 1024)
                        .then(|| iwi.level(0).get(..band).map(<[u8]>::to_vec))
                        .flatten()
                })
            }
            _ => None,
        },
        _ => None,
    };
    let line = format!(
        "t6 grade: vision {want} {}, colour table {} {}",
        if vision.is_some() { "read" } else { "MISSING" },
        image.map_or("(none)", |i| i.name.as_str()),
        if base.is_some() {
            "band 0"
        } else {
            "MISSING (identity)"
        },
    );
    // The scripts' `visionsetnaked` (mpOutro at a match's end, then this
    // map's own again) builds over the same table band.
    asset_world::set_t6_vision_base(base.clone());
    if let Some(v) = &vision {
        asset_world::register_t6_visions(vec![(map.to_owned(), *v)]);
    }
    let grade = vision.map(|vision| asset_world::T6Grade {
        lut: asset_world::t6_grade_lut(&vision, base.as_deref()),
        vision,
    });
    (grade, line)
}

/// (bo2mp's table had 30 lights with the fog in rows 30 and 31; the zombies
/// maps need more, so the fog rows now follow 256 lights.)
/// The light table: every primary light (a surface's light index is a
/// byte; Die Rise has 214, Origins 148), then the two fog rows.
const MAX_LIGHTS: usize = 256;
pub(crate) const SUN_FOG_SLOT: usize = MAX_LIGHTS;
pub(crate) const FOG_SLOT: usize = MAX_LIGHTS + 1;

/// Where a map's fog came from.
#[derive(Clone, Copy, Debug)]
enum T6Fog {
    World(asset_t6::WorldFogRef),
    Art(asset_t6::ArtFog),
}

/// The map's primary lights for the renderer's fallback draw, four vec4
/// each: origin and type (`ComPrimaryLight::type`: 1 sun, 2 spot, 5 omni),
/// colour and radius, direction and the cone's outer cosine, then
/// `dAttenuation` and the inner cosine. Slots 256 and 257 carry the fog
/// (`GfxWorldFog` terms): 257 = (start distance, half distance, opacity, 1),
/// (fog colour HDR, base height), (sun fog colour HDR, sun fog opacity),
/// (direction toward the sun fog, half height); 256 = the sun fog blend
/// t = saturate(y * dot(dir, view) + x) as (x, y, 0, 0).
fn t6_light_table(capture: &asset_t6::ZoneCapture, fog: Option<T6Fog>) -> Vec<[f32; 16]> {
    let mut table: Vec<[f32; 16]> = capture
        .primary_lights
        .iter()
        .take(MAX_LIGHTS)
        .map(|p| {
            [
                p.origin[0],
                p.origin[1],
                p.origin[2],
                f32::from(p.ty),
                p.color[0],
                p.color[1],
                p.color[2],
                p.radius,
                p.dir[0],
                p.dir[1],
                p.dir[2],
                p.cos_half_fov_outer,
                p.d_attenuation,
                p.cos_half_fov_inner,
                0.0,
                0.0,
            ]
        })
        .collect();
    let Some(fog) = fog else { return table };
    table.resize(FOG_SLOT + 1, [0.0; 16]);
    let [sun_row, fog_row] = fog_rows(fog);
    table[FOG_SLOT] = fog_row;
    table[SUN_FOG_SLOT] = sun_row;
    table
}

/// bo2mp: a Black Ops II world's own fog as the light table's two fog rows
/// (another game's map in its look, `bo2_look`).
pub(crate) fn world_fog_rows(world: &asset_t6::WorldRef) -> Option<[[f32; 16]; 2]> {
    world
        .init_fog
        .filter(|f| f.half_dist > 0.0 && f.fog_opacity > 0.0)
        .map(|f| fog_rows(T6Fog::World(f)))
}

/// The light table's two fog rows for a fog (slots 256 and 257, as
/// `t6_light_table` lays them out).
fn fog_rows(fog: T6Fog) -> [[f32; 16]; 2] {
    let (slot, sun) = match fog {
        T6Fog::World(f) => {
            // Pitch and yaw as the game's angles: pitch down is positive.
            let (p, y) = (f.sun_fog_pitch.to_radians(), f.sun_fog_yaw.to_radians());
            let dir = [p.cos() * y.cos(), p.cos() * y.sin(), -p.sin()];
            let (ci, co) = (
                f.sun_fog_inner.to_radians().cos(),
                f.sun_fog_outer.to_radians().cos(),
            );
            let scale = if (ci - co).abs() > 1e-4 {
                1.0 / (ci - co)
            } else {
                0.0
            };
            (
                [
                    f.base_dist,
                    f.half_dist,
                    f.fog_opacity,
                    1.0,
                    f.fog_color[0],
                    f.fog_color[1],
                    f.fog_color[2],
                    f.base_height,
                    f.sun_fog_color[0],
                    f.sun_fog_color[1],
                    f.sun_fog_color[2],
                    f.sun_fog_opacity,
                    dir[0],
                    dir[1],
                    dir[2],
                    // BO2: a half height under 1 is 1.
                    f.half_height.max(1.0),
                ],
                [-co * scale, scale],
            )
        }
        T6Fog::Art(f) => (
            [
                f.start_dist,
                f.half_dist,
                f.max_opacity,
                1.0,
                f.color[0] * f.scale,
                f.color[1] * f.scale,
                f.color[2] * f.scale,
                f.base_height,
                f.color[0] * f.scale,
                f.color[1] * f.scale,
                f.color[2] * f.scale,
                f.max_opacity,
                0.0,
                0.0,
                1.0,
                f.half_height,
            ],
            [0.0, 0.0],
        ),
    };
    let mut sun_row = [0.0; 16];
    sun_row[0] = sun[0];
    sun_row[1] = sun[1];
    [sun_row, slot]
}

/// The world's fog banks (the client scripts' `setworldfogactivebank`):
/// each fog volume's bank is its control word's low byte (Nuketown: three
/// bank-1 volumes with the map's fog, one bank-2 volume with none, which
/// its game-over shot switches to). Per bank, the first volume's rows.
fn fog_banks(world: &asset_t6::WorldRef) -> Vec<asset_world::T6FogVolume> {
    // Like an exposure volume's, a fog volume's control word holds its first
    // plane (bits 16..) and plane count less one (bits 8..15); its low byte
    // is the banks it belongs to, a bit mask (Origins' volumes 15, 7, 4, 2,
    // 1; Mob's csc sets banks 4, 8 and 1). Bank-less volumes never draw.
    world
        .fog_volumes
        .iter()
        .filter(|(_, _, control, _, _)| control & 0xff != 0)
        .map(|(_, _, control, _, fog)| {
            let start = (control >> 16) as usize;
            let count = ((control >> 8) & 0xff) as usize + 1;
            asset_world::T6FogVolume {
                planes: world
                    .fog_volume_planes
                    .get(start..start + count)
                    .map(<[_]>::to_vec)
                    .unwrap_or_default(),
                banks: control & 0xff,
                rows: fog_rows(T6Fog::World(*fog)),
            }
        })
        .collect()
}

/// The world's fog modifier volumes: their control words pick planes and
/// banks as fog volumes' do. Bank-less volumes never apply.
fn fog_modifiers(world: &asset_t6::WorldRef) -> Vec<asset_world::T6FogModifier> {
    world
        .fog_modifier_volumes
        .iter()
        .filter(|(control, _, _)| control & 0xff != 0)
        .map(|(control, scales, color_adjust)| {
            let start = (control >> 16) as usize;
            let count = ((control >> 8) & 0xff) as usize + 1;
            asset_world::T6FogModifier {
                planes: world
                    .fog_modifier_volume_planes
                    .get(start..start + count)
                    .map(<[_]>::to_vec)
                    .unwrap_or_default(),
                banks: control & 0xff,
                depth_scale: scales[1],
                color_adjust: *color_adjust,
            }
        })
        .collect()
}

/// The world's exposure volumes with their bounding planes: a volume's
/// control word holds its first plane (bits 16..) and its plane count less
/// one (bits 8..15) (measured: Nuketown 3 volumes of 10 planes at 20, 10
/// and 0; Tranzit 8 of 6; Mob's 8- and 9-plane volumes tile its 121).
fn exposure_volumes(world: &asset_t6::WorldRef) -> Vec<asset_world::T6ExposureVolume> {
    world
        .exposure_volumes
        .iter()
        .zip(&world.exposure_volume_controls)
        .map(|(v, control)| {
            let start = (control >> 16) as usize;
            let count = ((control >> 8) & 0xff) as usize + 1;
            asset_world::T6ExposureVolume {
                planes: world
                    .exposure_volume_planes
                    .get(start..start + count)
                    .map(<[_]>::to_vec)
                    .unwrap_or_default(),
                exposure: v[0],
                increase_scale: v[1],
                decrease_scale: v[2],
                feather_range: v[3],
            }
        })
        .collect()
}

/// The map's colour tables: the first image of its `lutMaterial` (RGBA8 in
/// the packs; Buried's `zm_buried_lut_win` is 1024x96, three tables).
fn world_lut(
    capture: &asset_t6::ZoneCapture,
    world: &asset_t6::WorldRef,
    packs: Option<&asset_t6::PackSet>,
) -> Result<asset_world::T6Lut, String> {
    let material = &capture.materials[world.lut_material.ok_or("no lutMaterial")?.index];
    let image = material
        .textures
        .first()
        .and_then(|t| t.image)
        .map(|k| &capture.images[k.index])
        .ok_or("lutMaterial has no image")?;
    let Some(asset_t6::ImageSource::Pack(i, entry)) = packs.and_then(|p| p.locate(image)) else {
        return Err(format!("{} is in no pack", image.name));
    };
    let bytes = packs.ok_or("no packs")?.packs[i].read(entry)?;
    let iwi = ipak_t6::parse_iwi(&bytes).map_err(|e| e.to_string())?;
    let rgba = iwi.level(0);
    if iwi.format != ipak_t6::IwiFormat::Rgba8
        || iwi.width != 1024
        || iwi.height % 32 != 0
        || rgba.len() < (iwi.width * iwi.height * 4) as usize
    {
        return Err(format!("{} is {:?} {}x{}", image.name, iwi.format, iwi.width, iwi.height));
    }
    Ok(asset_world::T6Lut {
        width: iwi.width,
        height: iwi.height,
        rgba: rgba[..(iwi.width * iwi.height * 4) as usize].to_vec(),
    })
}

/// The volumes that pick among the colour tables, as the game stores them
/// (`control`: banks in the low byte, the plane count less one in the next,
/// the first plane above).
fn lut_volumes(world: &asset_t6::WorldRef) -> Vec<asset_world::T6LutVolume> {
    world
        .lut_volumes
        .iter()
        .map(|&(control, table, transition_time)| {
            let start = (control >> 16) as usize;
            let count = ((control >> 8) & 0xff) as usize + 1;
            asset_world::T6LutVolume {
                planes: world
                    .lut_volume_planes
                    .get(start..start + count)
                    .map(<[_]>::to_vec)
                    .unwrap_or_default(),
                banks: control & 0xff,
                table: table & 0x1f,
                transition_time,
            }
        })
        .collect()
}

impl ZoneLane for T6Lane {
    fn game(&self) -> ZoneGame {
        Self::GAME
    }

    fn capabilities(&self) -> &'static [(PreparedCapability, LaneStatus)] {
        Self::CAPABILITIES
    }

    fn load_world(
        &self,
        path: &Path,
        image: &ZoneImage,
        progress: &LoadProgress,
        _shared_surfaces: asset_model::SharedXModelSurfaces,
        material_seed: asset_material::MaterialCatalog,
        _common_film_visions: &mut std::collections::BTreeMap<
            String,
            Result<asset_world::FilmVision, asset_world::FilmVisionParseError>,
        >,
    ) -> LoadedWorld {
        let mut report = vec![
            format!("game: T6 ({})", path.display()),
            asset_census(path, image),
        ];
        let stage = progress.begin_scoped(StageId::MapAssets, "walk", None);
        let capture = match asset_t6::capture_image(path, image) {
            Ok(capture) => {
                stage.done();
                capture
            }
            Err(error) => {
                stage.fail();
                return LoadedWorld::with_gap(
                    WorldDrawPolicy::t5(),
                    PreparedCapability::PreparedWorld,
                    format!("T6 zone walk: {error}"),
                    Some("assets::lane::t6::load_world/walk"),
                );
            }
        };
        report.push(format!(
            "t6 capture: {} images, {} materials, {} technique sets, {} xmodels, {} unresolved refs",
            capture.images.len(),
            capture.materials.len(),
            capture.technique_sets.len(),
            capture.xmodels.len(),
            capture.unresolved_refs
        ));

        let map = path
            .file_stem()
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
        let (origin, yaw) = player_start(&capture, &map, &mut report);
        let scripts = super::t6_scripts::start_scripts(origin, yaw, map.starts_with("zm_"));

        let collision = match capture
            .clip
            .as_ref()
            .map(asset_world::build_t6_clip_collision)
        {
            Some(Ok(mut clip)) => {
                let (placed, with_tris) = attach_t6_static_models(&capture, &mut clip);
                report.push(format!(
                    "t6 clip static models: {placed} placed, {with_tris} with collision triangles"
                ));
                let world_brushes = clip.cmodels.first().map_or(0, |m| m.num_brushes);
                report.push(format!(
                    "t6 clip: {} brushes ({} in the world model), {} nodes, {} leaves, {} brush models, {} mesh verts, {} mesh tris, {} materials",
                    clip.brushes.len(),
                    world_brushes,
                    clip.nodes.len(),
                    clip.leaves.len(),
                    clip.cmodels.len(),
                    clip.mesh.verts.len(),
                    clip.mesh.tri_indices.len() / 3,
                    clip.materials.len()
                ));
                Some(clip)
            }
            Some(Err(error)) => {
                report.push(format!("t6 clip gap: {error}"));
                None
            }
            None => {
                report.push("t6 clip gap: zone has no clipMap".to_owned());
                None
            }
        };

        let spawns = vec![asset_world::SpawnPoint {
            classname: "mp_dm_spawn".to_owned(),
            origin,
            angles: [0.0, yaw, 0.0],
            script_linkto: String::new(),
            script_destructable_area: String::new(),
        }];

        let Some(world) = capture.world.as_ref() else {
            let mut loaded = LoadedWorld::with_gap(
                WorldDrawPolicy::t5(),
                PreparedCapability::PreparedWorld,
                "T6 zone has no GfxWorld",
                Some("assets::lane::t6::load_world/no_world"),
            );
            loaded.scripts = scripts;
            loaded.collision = collision;
            loaded.spawns = spawns;
            loaded.report.extend(report);
            return loaded;
        };
        // Materials: every world surface's, linked into the lane's catalog
        // with its colour map decoded from the packs beside the zone.
        let stage = progress.begin_scoped(StageId::MapAssets, "materials", None);
        // IW4L_HEADLESS=1: nothing is drawn, so no colour map is read or
        // decoded; the materials still link, without their images.
        let packs = match Some(asset_transport::t6_extra::install_zone_dir(path))
            .filter(|_| !frame::Headless::requested())
            .map(|dir| asset_t6::PackSet::open_dir(&dir))
        {
            Some(Ok(packs)) => Some(packs),
            Some(Err(error)) => {
                report.push(format!("t6 image packs: {error}"));
                None
            }
            None => None,
        };
        let mut materials = material_seed;
        materials.set_capture_ns(asset_core::AssetNamespace::T6);
        let sky_materials = capture
            .xmodels
            .iter()
            .filter(|m| m.name == world.sky_box_model)
            .flat_map(|m| m.lod0.iter())
            .filter_map(|srf| srf.material.map(|k| k.index));
        let model_materials = world
            .static_models
            .iter()
            .filter_map(|sm| sm.model)
            .flat_map(|k| capture.xmodels[k.index].lod0.iter())
            .filter_map(|srf| srf.material.map(|k| k.index))
            .chain(sky_materials);
        let mut areas = AreaModels::read(path, &map, &capture, world);
        let area_map_materials: Vec<usize> = areas
            .surface_materials()
            .filter_map(|(ci, mi)| ci.is_none().then_some(mi))
            .collect();
        let linked = super::t6_materials::link_materials(
            &mut materials,
            &capture,
            packs.as_ref(),
            world
                .surfaces
                .iter()
                .filter_map(|s| s.material.map(|k| k.index))
                .chain(model_materials)
                .chain(area_map_materials),
            &map,
        );
        stage.done();
        report.extend(linked.report);
        areas.link(&mut materials, &capture, packs.as_ref(), &mut report);
        let surface_materials: Vec<Option<usize>> = world
            .surfaces
            .iter()
            .map(|s| s.material.and_then(|k| linked.local.get(&k.index).copied()))
            .collect();
        // Lighting: each lightmap's secondary image (the only one T6 keeps),
        // the world's sun, and its exposure (exposure values are stops: the
        // scale is 2^-exposure).
        let lightmaps: Vec<Option<asset_world::WorldLightmap>> = world
            .lightmaps
            .iter()
            .map(|pair| {
                pair[1]
                    .and_then(|k| capture.images.get(k.index))
                    .and_then(asset_world::t6_lightmap)
            })
            .collect();
        // bo2zm fix list 2: the same pages on the CPU, for decals on walls.
        asset_world::install_t6_lightmaps(
            world
                .lightmaps
                .iter()
                .map(|pair| {
                    let image = pair[1].and_then(|k| capture.images.get(k.index))?;
                    let embedded = image.embedded.as_ref().filter(|e| e.dxgi_format == 28)?;
                    let (width, height) = (u32::from(image.width), u32::from(image.height));
                    let rgba = embedded.data.get(..(width * height * 4) as usize)?.to_vec();
                    Some(asset_world::T6LightmapPage {
                        width,
                        height,
                        rgba,
                    })
                })
                .collect(),
        );
        // Two colour grades meet here: bo2mp's (the vision and colour table
        // band 0 built into one final-pass table, `install_t6_grade`; the
        // scripts' `visionsetnaked` switches it) and bo2zm's (the vision's
        // grade in the world lighting, the colour table volumes and the
        // scripts' visions). A multiplayer map keeps bo2mp's, a Zombies map
        // bo2zm's; never both (that grades twice).
        let mp_grade = super::t6_m2::is_mp_map(&map);
        let exposure = if mp_grade {
            // bo2mp: a map with no exposure volume (Cove, Plaza, Pod, Uplink)
            // takes its world sun's own exposure (`initWorldSun.exposure`), not
            // 0 stops (drawn four to eight times too bright).
            world
                .exposure_volumes
                .first()
                .map_or(world.init_sun_exposure, |v| v[0])
        } else {
            // bo2zm: the exposure outside every exposure volume: the map's
            // sun's (`sunParse.initWorldSun`, its last word: Nuketown 4.8,
            // Tranzit 3, Origins 3 stops). Origins has no exposure volumes at
            // all, so without it it drew at 0 stops, 8x too bright.
            world
                .init_sun_raw
                .get(20)
                .map(|w| f32::from_bits(*w))
                .filter(|e| e.is_finite() && *e > 0.0 && *e < 16.0)
                .or_else(|| world.exposure_volumes.first().map(|v| v[0]))
                .unwrap_or(0.0)
        };
        if mp_grade {
            let (grade, grade_line) = t6_grade(&capture, world, packs.as_ref(), &map);
            asset_world::install_t6_grade(grade);
            report.push(grade_line);
        } else {
            // (A multiplayer map's final-pass grade does not stay on.)
            asset_world::install_t6_grade(None);
        }
        // The sky: its material's rotation and brightness constants.
        let sky_material = capture
            .xmodels
            .iter()
            .find(|m| m.name == world.sky_box_model)
            .and_then(|m| m.lod0.first())
            .and_then(|srf| srf.material)
            .map(|k| &capture.materials[k.index]);
        let constant = |name: &str| {
            sky_material
                .and_then(|m| m.constants.iter().find(|c| name.starts_with(&c.1)))
                .map(|c| c.2)
        };
        let rotation = constant("skyBoxRotation").unwrap_or([1.0, 0.0, -0.0, 1.0]);
        let brightness = constant("skyColorParm").map_or(1.0, |c| c[1]);
        let sky_parms = constant("skyColorParm").map_or([1.0, 1.0], |c| [c[0], c[3]]);
        // `r_skyTransition` blends the sky shader's two cube lookups, the
        // second with z flipped the other way. Die Rise's client script sets
        // it to 1 (`clientscripts/mp/zm_highrise.csc` main), so it draws the
        // cube's other half: the dark storm, not the blue-white day.
        let sky_transition = map == "zm_highrise";
        let sky = [
            rotation[0],
            rotation[1],
            rotation[2],
            if (rotation[3] < 0.0) != sky_transition {
                -brightness
            } else {
                brightness
            },
        ];
        // The map's vision (`vision/<map>.vision`; Origins starts on
        // zm_tomb): the final image's saturation and colour grade. Die Rise's
        // client script picks its vision by fog volume with the power's
        // suffix (`fog_vol_to_visionset_set_suffix("_off")` in
        // `clientscripts/mp/zm_highrise.csc`), so it starts on
        // zm_highrise_off; its plain vision is an identity grade.
        let vision = if map == "zm_highrise" {
            format!("vision/{map}_off.vision")
        } else {
            format!("vision/{map}.vision")
        };
        let vision_bytes = capture
            .raw_files
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&vision))
            .map(|(_, bytes)| bytes.as_slice());
        // (bo2mp: a multiplayer map's grade is its final-pass table above.)
        let saturation = vision_bytes.filter(|_| !mp_grade).map(vision_saturation);
        let grade = vision_bytes.filter(|_| !mp_grade).map(vision_grade);
        report.push(format!("t6 vision: {vision} saturation {saturation:?} grade {grade:?}"));
        let lighting = world.sun.map(|sun| asset_world::T6WorldLighting {
            sun: asset_world::WorldSunLight {
                direction: sun.dir,
                diffuse: [sun.color[0], sun.color[1], sun.color[2], 1.0],
                specular: [sun.color[0], sun.color[1], sun.color[2], 1.0],
            },
            exposure_scale: (-exposure).exp2(),
            sky,
            sky_parms,
            saturation,
            grade,
        });
        report.push(format!(
            "t6 lighting: {} of {} lightmaps built, sun {:?}, exposure {exposure} (scale {:.4})",
            lightmaps.iter().filter(|l| l.is_some()).count(),
            lightmaps.len(),
            world.sun.map(|s| s.color),
            (-exposure).exp2()
        ));
        // Layered surfaces: each one's extra layers and their second-stream
        // layout, from its technique set's name.
        let surface_layers: Vec<asset_core::T6Layers> = world
            .surfaces
            .iter()
            .map(|s| {
                s.material
                    .and_then(|k| capture.materials[k.index].technique_set)
                    .map(|k| {
                        asset_core::T6Layers::from_technique_set(
                            &capture.technique_sets[k.index].name,
                        )
                    })
                    .unwrap_or_default()
            })
            .collect();
        report.push(format!(
            "t6 layered surfaces: {} of {}",
            surface_layers.iter().filter(|l| l.any()).count(),
            surface_layers.len()
        ));
        match asset_world::build_t6_world_draw(
            world,
            &surface_materials,
            &surface_layers,
            lightmaps,
            lighting,
        ) {
            Ok(mut draw) => {
                // The map's fog: its own initial world fog (what the game
                // draws; the client script picks among its fog banks), else
                // its createart script's setvolfog.
                let fog = world
                    .init_fog
                    .filter(|f| f.half_dist > 0.0 && f.fog_opacity > 0.0)
                    .map(T6Fog::World)
                    .or_else(|| art_fog(&capture, path, &map).map(T6Fog::Art));
                report.push(format!("t6 fog: {fog:?}"));
                draw.t6_lights = t6_light_table(&capture, fog);
                draw.t6_fog_banks = fog_banks(world);
                draw.t6_fog_modifiers = fog_modifiers(world);
                report.push(format!(
                    "t6 fog modifiers: {:?}",
                    draw.t6_fog_modifiers
                        .iter()
                        .map(|m| (m.banks, m.planes.len(), m.depth_scale, m.color_adjust))
                        .collect::<Vec<_>>()
                ));
                draw.t6_exposure_volumes = exposure_volumes(world);
                // The map's own colour tables, graded before its vision
                // (`hdr_create_lut2dv`), and the volumes picking among them.
                // (bo2mp: a multiplayer map's colour table is in its
                // final-pass grade above.)
                match world_lut(&capture, world, packs.as_ref()).and_then(|lut| {
                    if mp_grade {
                        Err("multiplayer map: graded by its final-pass table".to_owned())
                    } else {
                        Ok(lut)
                    }
                }) {
                    Ok(lut) => {
                        report.push(format!(
                            "t6 colour table: {}x{} ({} tables)",
                            lut.width,
                            lut.height,
                            lut.height / 32
                        ));
                        draw.t6_lut = Some(std::sync::Arc::new(lut));
                    }
                    Err(e) => report.push(format!("t6 colour table: none ({e})")),
                }
                if !mp_grade {
                    draw.t6_lut_volumes = lut_volumes(world);
                }
                // The sun light and colour the client scripts'
                // r_lightTweakSunLight / SunColor replace (Buried 13, TranZit
                // 12, Nuketown 15; Origins 0.38 0.78 1) and the sky intensity
                // r_sky_intensity_* replaces.
                draw.t6_light_tweaks = world
                    .init_sun_raw
                    .get(11)
                    .map(|w| f32::from_bits(*w))
                    .filter(|v| v.is_finite() && *v > 0.0)
                    .map(|sun_light| asset_world::T6LightTweaks {
                        sun_light,
                        sun_color: [8, 9, 10].map(|i| {
                            world.init_sun_raw.get(i).map_or(1.0, |w| f32::from_bits(*w))
                        }),
                        sky_intensity: world.sky_dyn_intensity,
                    });
                report.push(format!("t6 light tweaks: {:?}", draw.t6_light_tweaks));
                report.push(format!(
                    "t6 colour table volumes: {:?}",
                    draw.t6_lut_volumes
                        .iter()
                        .map(|v| (v.table, v.banks, v.transition_time, v.planes.len()))
                        .collect::<Vec<_>>()
                ));
                report.push(format!(
                    "t6 exposure volumes: {:?}",
                    draw.t6_exposure_volumes
                        .iter()
                        .map(|v| (v.exposure, v.planes.len()))
                        .collect::<Vec<_>>()
                ));
                report.push(format!(
                    "t6 fog banks: {:?}",
                    draw.t6_fog_banks
                        .iter()
                        .map(|v| (v.banks, v.planes.len(), v.rows[1][2]))
                        .collect::<Vec<_>>()
                ));
                // Reflection probes: their cube maps (one cube array) and
                // the lighting each was captured in.
                draw.t6_probes = world.reflection_probes.iter().map(|p| p.lighting_sh).collect();
                asset_world::install_t6_probe_origins(
                    world.reflection_probes.iter().map(|p| p.origin).collect(),
                );
                if let Some(line) =
                    super::t6_materials::link_reflection_probes(&mut materials, &capture, world, &map)
                {
                    report.push(line);
                }
                report.push(format!(
                    "t6 primary lights: {} (types {:?})",
                    draw.t6_lights.len(),
                    capture
                        .primary_lights
                        .iter()
                        .map(|p| p.ty)
                        .collect::<Vec<_>>()
                ));
                report.push(format!(
                    "world mesh: {} vertices, {} triangles, {} surfaces (fallback draw with colour maps)",
                    draw.stats.vertices, draw.stats.triangles, draw.stats.surfaces
                ));
                let (min, max) = (draw.stats.min, draw.stats.max);
                let props = build_props(&capture, world, &linked.local, &areas, collision.as_ref(), &mut report);
                // bo2mp: Black Ops III's look (`BO2MP_BO3_LOOK=1`; off, the
                // map is drawn as before).
                if super::bo3_look::on()
                    && let Some(sun) = world.sun.as_ref()
                {
                    let spawn_points: Vec<[f32; 3]> = capture
                        .map_ents
                        .iter()
                        .flat_map(|text| asset_t6::parse_entities(text))
                        .filter(|e| {
                            let c = e.classname();
                            c.starts_with("mp_") && c.contains("_spawn")
                        })
                        .filter_map(|e| e.vec3("origin"))
                        .collect();
                    // (A few, to place cameras by.)
                    let some: Vec<String> = spawn_points.iter().step_by((spawn_points.len() / 6).max(1)).take(6).map(|p| format!("{:.0} {:.0} {:.0}", p[0], p[1], p[2])).collect();
                    report.push(format!("bo3 look: {} spawn points, some at {}", spawn_points.len(), some.join(", ")));
                    super::bo3_look::apply(
                        &mut draw,
                        &materials,
                        &props.0,
                        &props.1,
                        &spawn_points,
                        sun.dir,
                        sun.color,
                        &mut report,
                    );
                }
                // bo2zm M2: guns, arms, animations, effects and sound from
                // every zone the map loads.
                let stage = progress.begin_scoped(StageId::MapAssets, "weapons, effects, sound", None);
                let combat =
                    super::t6_m2::load_t6_combat(path, &capture, &mut materials, packs.as_ref());
                stage.done();
                report.extend(combat.report);
                // The visions a script turns on (Mob's Afterlife), graded
                // as the map's own.
                draw.t6_script_grades = combat
                    .visions
                    .iter()
                    .map(|(name, bytes)| (name.clone(), vision_grade(bytes)))
                    .collect();
                report.push(format!(
                    "t6 script visions: {}",
                    draw.t6_script_grades
                        .iter()
                        .map(|(name, _)| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                let prepare = script_sound_aliases(&combat.scripts);
                report.push(format!("t6 script sounds prepared at load: {}", prepare.len()));
                let facts = crate::MapFacts {
                    t6_footsteps: combat.footsteps,
                    t6_hud_icons: combat.hud_icons,
                    t6_hud_fonts: combat.hud_fonts,
                    t6_ui: combat.ui,
                    t6_playeranim: combat.scripts.playeranim.clone(),
                    t6_ragdoll: combat.scripts.ragdoll.clone(),
                    t6_anim_speeds: combat
                        .scripts
                        .anims
                        .iter()
                        .filter(|a| a.name.starts_with("pb_") || a.name.starts_with("pl_"))
                        .filter_map(|a| {
                            let len = f32::from(a.numframes) / a.framerate;
                            let (first, last) = (a.delta_trans.first()?.1, a.delta_trans.last()?.1);
                            let v = [last[0] - first[0], last[1] - first[1], last[2] - first[2]];
                            // T5 BG_AnimParseAnimScript: an animation that mostly
                            // rises (x + y <= 0.8 z, a ladder climb up) moves by its
                            // full 3D delta, every other one by its ground delta.
                            let d = if v[0] + v[1] <= v[2] * 0.8 {
                                (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
                            } else {
                                (v[0] * v[0] + v[1] * v[1]).sqrt()
                            };
                            (len > 0.0 && d > 1.0).then(|| (a.name.to_ascii_lowercase(), d / len))
                        })
                        .collect(),
                    t6_pen_table: combat.pen_table,
                    t6_screen_filters: combat.screen_filters,
                    script_sound: asset_audio::MapScriptSoundFacts {
                        ambient_alias: combat.room_tone,
                        script: Some(format!("clientscripts/mp/{map}_amb.csc")),
                        prepare,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let mut scripts = scripts;
                scripts.t6 = Some(combat.scripts);
                LoadedWorld {
                    scripts,
                    world: PreparedWorld {
                        draw: Some(draw),
                        min,
                        max,
                        policy: WorldDrawPolicy::t5(),
                        static_model_meshes: props.0,
                        static_model_instances: props.1,
                        fx: combat.fx,
                        fx_models: combat.fx_models,
                        impact_fx: combat.impact_fx,
                        ..Default::default()
                    },
                    materials,
                    collision,
                    spawns,
                    fpv_meshes: combat.fpv,
                    xanims: combat.xanims,
                    weapons: combat.weapons,
                    world_weapons: combat.world_weapons,
                    tracers: combat.tracers,
                    pen_table: combat.pen_table,
                    sound: Some(combat.sound),
                    facts,
                    report,
                    ..Default::default()
                }
            }
            Err(error) => {
                let mut loaded = LoadedWorld::with_gap(
                    WorldDrawPolicy::t5(),
                    PreparedCapability::PreparedWorld,
                    format!("T6 world mesh: {error}"),
                    Some("assets::lane::t6::load_world/world_mesh"),
                );
                loaded.scripts = scripts;
                loaded.collision = collision;
                loaded.spawns = spawns;
                loaded.report.extend(report);
                loaded
            }
        }
    }

    fn load_common_mp(
        &self,
        path: &Path,
        image: &ZoneImage,
        _progress: &LoadProgress,
        _decode_color_maps: bool,
        _material_seed: asset_material::MaterialCatalog,
    ) -> CommonCensus {
        CommonCensus {
            report: vec![
                format!(
                    "Black Ops II common assets are not read yet (bo2zm M1): {}",
                    path.display()
                ),
                asset_census(path, image),
            ],
            ..Default::default()
        }
    }

    fn load_material_population(
        &self,
        path: &Path,
        image: &ZoneImage,
        _progress: &LoadProgress,
        material_seed: asset_material::MaterialCatalog,
    ) -> MaterialPopulation {
        MaterialPopulation {
            materials: material_seed,
            report: vec![
                format!(
                    "Black Ops II materials are not read yet (bo2zm M1): {}",
                    path.display()
                ),
                asset_census(path, image),
            ],
            ..Default::default()
        }
    }
}

/// bo2mp: the map Black Ops II's own rules borrow the faction pairing and
/// mapstable row of, for a map from another game (Aftermath: SEALs and
/// the PMC).
const FOREIGN_ROW_FROM: &str = "mp_la";

/// bo2mp: a foreign map's surfaces in BO2's list (footsteps, impacts and
/// penetration read them so under BO2's rules): an Infinity Ward map's
/// (`iw_list`: MW2, MW3, CoD4) slush and riot shield moved to BO2's own;
/// Black Ops' and World at War's lists are BO2's already. Logs how many of
/// each surface the map's collision has, to check footsteps by.
pub(crate) fn t6_surface_list(loaded: &mut LoadedWorld, map: &str, iw_list: bool) {
    let Some(clip) = loaded.collision.as_mut() else {
        return;
    };
    let moved = if iw_list {
        asset_world::remap_clip_surface_flags(clip, asset_world::iw_surface_flags_to_t6)
    } else {
        0
    };
    let census = asset_world::clip_surface_type_census(clip);
    let names = weapon_iw4::T6_SURFACE_TYPE_NAMES;
    let mut rows: Vec<(usize, &str)> = census
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| (*n, names.get(i).copied().unwrap_or("?")))
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    let line = format!(
        "bo2 rules: {map} surfaces in BO2's list ({moved} moved{}): {}",
        if iw_list { " from Infinity Ward's" } else { "" },
        rows.iter()
            .map(|(n, name)| format!("{name}={n}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    diag::info!(World, "{line}");
    loaded.report.push(line);
}

/// bo2mp: what lights Black Ops II's moving things (the first-person gun
/// and arms, players, grenades, marks) on a map from another game. A Black
/// Ops II map installs these in its own lane; a foreign one installed none,
/// so the gun and players read the previous Black Ops II map's light grid
/// at this map's points, or none (black), and the fallback shader's
/// primary-light table was empty (no sun on them). Here: the map's own
/// light grid read as Black Ops II light, its primary lights in the
/// table's layout (sun 1, spot 2, omni 5), and the last Black Ops II
/// map's lightmaps and probe origins dropped.
pub(super) fn foreign_t6_lighting(loaded: &mut LoadedWorld, map: &str) {
    let lights: Vec<[f32; 16]> = loaded.world.draw.as_ref().map_or_else(Vec::new, |draw| {
        draw.primary_lights
            .iter()
            .take(SUN_FOG_SLOT)
            .map(|p| {
                let kind = match p.light_type {
                    1 => 1.0,
                    2 => 2.0,
                    3 => 5.0,
                    _ => 0.0,
                };
                [
                    p.origin[0],
                    p.origin[1],
                    p.origin[2],
                    kind,
                    p.color[0],
                    p.color[1],
                    p.color[2],
                    p.radius,
                    p.direction[0],
                    p.direction[1],
                    p.direction[2],
                    p.cos_outer,
                    // Full strength inside its radius (the shader's
                    // dAttenuation / d^2, clamped), fading to its edge.
                    p.radius * p.radius,
                    p.cos_inner,
                    0.0,
                    0.0,
                ]
            })
            .collect()
    });
    let sun = lights.iter().position(|l| l[3] == 1.0);
    // Only where the world itself is drawn by its own game's path (MW2,
    // MW3: no exposure carrier). A Black Ops or CoD4 world is drawn by the
    // fallback pass, whose world surfaces read this table too; theirs stays
    // as it was.
    let table = loaded
        .world
        .draw
        .as_mut()
        .filter(|draw| draw.t6_lights.is_empty() && draw.t5_sun_parse_exposure.is_none());
    let table_filled = table.is_some();
    if let Some(draw) = table {
        draw.t6_lights = lights.clone();
    }
    let grid = loaded.world.light_grid.clone();
    let sun_primary = grid
        .as_ref()
        .map(|g| g.sun_primary_light_index as u8)
        .filter(|&i| i != 0)
        .or(sun.map(|i| i as u8))
        .unwrap_or(0);
    // bo2mp: a CoD4 / World at War map (drawn by the fallback pass's IW3
    // branch) lights its models as those games do (`lp_sun_*`: twice the
    // grid's colour plus the sun by N.L times its visibility, in gamma
    // space; the pass's `add_light`): the grid's sun row in the light table
    // is the sun, and the models' sun term is on.
    // (Ghosts and Infinite Warfare draw by that branch too, without a grid:
    // theirs stay as they were.)
    let legacy = grid.is_some() && loaded.world.draw.as_ref().is_some_and(|d| d.iw3_lightmaps);
    if legacy
        && sun_primary != 0
        && usize::from(sun_primary) < SUN_FOG_SLOT
        && let Some(draw) = loaded.world.draw.as_mut()
    {
        if draw.t6_lights.len() < 32 {
            draw.t6_lights.resize(32, [0.0; 16]);
        }
        draw.t6_lights[usize::from(sun_primary)][3] = 1.0;
    }
    let scale = match grid.as_ref().map(|g| g.color_encoding) {
        Some(asset_model::LightGridColorEncoding::T5Y12U6W6) => 255.0 / 8.0,
        _ => 4.0,
    };
    // bo2mp: the primary light's term on models is off on another game's
    // map (`BO2MP_FOREIGN_SUN=1` brings it back): see `sample_foreign`.
    let foreign_sun = legacy || std::env::var("BO2MP_FOREIGN_SUN").is_ok_and(|v| v.trim() == "1");
    asset_world::set_foreign_sun(foreign_sun);
    let foreign = asset_world::ForeignGridLight {
        scale,
        outside: asset_world::T6PointLight {
            light: [0.5; 3],
            primary: sun_primary,
            visibility: if sun_primary != 0 && foreign_sun { 1.0 } else { 0.0 },
        },
    };
    let sun_colour = sun.and_then(|i| lights.get(i)).map(|l| [l[4], l[5], l[6]]);
    let line = format!(
        "bo2 rules: {map} moving-thing lighting: sun colour {sun_colour:?}, models' sun term {}; light grid {} (scale {scale:.2}), {} primary lights (sun {:?}, grid sun {sun_primary}, table {}), collision {}",
        if foreign_sun { "on" } else { "off" },
        if grid.is_some() { "READY" } else { "absent (ambient + sun)" },
        lights.len(),
        sun,
        if table_filled { "filled" } else { "kept" },
        loaded.collision.is_some(),
    );
    asset_world::install_t6_light_sampler(Some(asset_world::T6LightSampler {
        mins: grid.as_ref().map_or([0; 3], |g| g.mins),
        maxs: grid.as_ref().map_or([0; 3], |g| g.maxs),
        grid,
        sky: Vec::new(),
        scale: 1.0,
        sun_primary,
        clip: loaded.collision.as_ref().map(|c| std::sync::Arc::new(c.clone())),
        cache: Default::default(),
        foreign: Some(foreign),
    }));
    asset_world::install_t6_lightmaps(Vec::new());
    asset_world::install_t6_probe_origins(Vec::new());
    diag::info!(World, "{line}");
    loaded.report.push(line);
}

/// bo2mp: Black Ops II's rules on a map from another game (MW2, MW3, Black
/// Ops). The map's own lane has walked its world, collision, entities and
/// ambience into `loaded`; Black Ops II's guns, arms, animations, effects,
/// sound, scripts, HUD and factions are added from the Black Ops II zones
/// (as a BO2 map would load them), and the map's own scripts are put aside
/// for BO2's start-up roots, as on a BO2 map.
pub(crate) fn attach_t6_rules(
    loaded: &mut LoadedWorld,
    map: &str,
    source: asset_world::ForeignEntityGame,
) {
    foreign_t6_lighting(loaded, map);
    // No Black Ops II vision for this map: its own game's film vision when
    // it has one (CoD4's, World at War's: drawn as the colour table of
    // Black Ops II's final pass), else its colours as they are. The
    // scripts' `visionsetnaked <map>` after `mpIntro` (greyscale) comes back
    // to that, not to the grey (nor to a Black Ops II map's grade before it).
    if let Some(grade) = super::bo2_look::take_pending_grade() {
        asset_world::install_t6_grade(Some(grade));
        loaded.report.push(format!("bo2 rules grade: {map} graded as its Black Ops II model map"));
    } else if asset_world::install_film_map_grade(loaded.world.film_vision.as_ref()) {
        loaded.report.push(format!("bo2 rules grade: {map} graded by its own game's film vision"));
    } else {
        loaded.report.push(format!("bo2 rules grade: {map} has no Black Ops II vision; drawn ungraded after mpIntro"));
    }
    let mut report = Vec::new();
    let dir = match asset_transport::games_root_from_env()
        .and_then(|root| asset_transport::find_zone_file(&root, "t6:patch_mp"))
    {
        Ok(found) => match found.path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                loaded.report.push("bo2 rules gap: patch_mp has no folder".to_owned());
                return;
            }
        },
        Err(error) => {
            loaded
                .report
                .push(format!("bo2 rules gap: no Black Ops II zones to borrow from ({error})"));
            return;
        }
    };
    // Black Ops II's combat zones load as a BO2 map's would: beside a map
    // of this name in Black Ops II's zone folder (the map's own zone is
    // not there; its row and factions are borrowed below).
    let path = dir.join(format!("{map}.ff"));
    let packs = if frame::Headless::requested() {
        None
    } else {
        match asset_t6::PackSet::open_dir(&dir) {
            Ok(packs) => Some(packs),
            Err(error) => {
                report.push(format!("bo2 rules: t6 image packs: {error}"));
                None
            }
        }
    };
    let previous_ns = loaded.materials.capture_ns();
    loaded.materials.set_capture_ns(asset_core::AssetNamespace::T6);
    // bo2mp: a burning World at War map's fires and smoke: Black Ops II's
    // from its model map's zone (only those effects of it), in the map's
    // place among the zones.
    let pending_fx = super::bo2_look::take_pending_fx();
    let empty = match &pending_fx {
        Some((shots, zone)) if !shots.is_empty() => {
            let wanted = shots.iter().map(|s| s.fxid.clone()).collect();
            match super::bo2_look::donor_fx_capture(&dir, zone, &wanted) {
                Ok(c) => {
                    report.push(format!("bo2 rules: {} effects from {zone} for the map's fires and smoke", c.fx.len()));
                    c
                }
                Err(e) => {
                    report.push(format!("bo2 rules: {zone} effects not read ({e})"));
                    asset_t6::ZoneCapture::default()
                }
            }
        }
        _ => asset_t6::ZoneCapture::default(),
    };
    let mut combat = super::t6_m2::load_t6_combat(&path, &empty, &mut loaded.materials, packs.as_ref());
    if let (Some((shots, _)), Ok(sound)) = (pending_fx, combat.sound.as_mut()) {
        let n = shots.len();
        sound.scripted_map_fx.get_or_insert_with(Default::default).oneshots = shots;
        report.push(format!("bo2 rules: {n} map effects placed"));
    }
    loaded.materials.set_capture_ns(previous_ns);
    report.append(&mut combat.report);

    // The map's entities (spawns, objectives, doors) for BO2's scripts,
    // with BO2's names for the game types' spawn classes and objectives
    // (block for block: the BO2 scripts' map entities are matched back to
    // the map's own drawn models and brushes by their place in the text).
    let foreign = asset_world::foreign_entities_for_t6_rules(
        loaded.scripts.entities().unwrap_or_default(),
        source,
    );
    report.extend(foreign.report);
    combat.scripts.entities.push(foreign.text);
    // Its row in BO2's map table: the borrowed map's, under its own name
    // (BO2's scripts read the factions and the map's name from it).
    if let Some((_, columns, rows, cells)) = combat
        .scripts
        .tables
        .iter_mut()
        .find(|(name, ..)| name.eq_ignore_ascii_case("mp/mapstable.csv"))
    {
        let columns = *columns;
        let has_own = (0..*rows).any(|r| cells.get(r * columns).is_some_and(|c| c.eq_ignore_ascii_case(map)));
        let borrowed = (0..*rows)
            .find(|&r| cells.get(r * columns).is_some_and(|c| c.eq_ignore_ascii_case(FOREIGN_ROW_FROM)))
            .map(|r| cells[r * columns..((r + 1) * columns).min(cells.len())].to_vec());
        if let (false, Some(mut row)) = (has_own, borrowed) {
            row.resize(columns, String::new());
            row[0] = map.to_owned();
            cells.extend(row);
            *rows += 1;
            report.push(format!("bo2 rules: mapstable row for {map} borrowed from {FOREIGN_ROW_FROM}"));
        }
    }

    // The start-up roots BO2's maps run with, at the map's first spawn.
    let (origin, yaw) = loaded
        .spawns
        .first()
        .map_or(([0.0; 3], 0.0), |s| (s.origin, s.angles[1]));
    let mut scripts = super::t6_scripts::start_scripts(origin, yaw, false);
    scripts.insert_source(&format!("maps/mp/{map}"), "main()\n{\n}\n".to_owned());

    loaded.facts.t6_footsteps = combat.footsteps;
    loaded.facts.t6_hud_icons = combat.hud_icons;
    loaded.facts.t6_hud_fonts = combat.hud_fonts;
    loaded.facts.t6_ui = combat.ui;
    loaded.facts.t6_playeranim = combat.scripts.playeranim.clone();
    loaded.facts.t6_ragdoll = combat.scripts.ragdoll.clone();
    loaded.facts.t6_anim_speeds = combat
        .scripts
        .anims
        .iter()
        .filter(|a| a.name.starts_with("pb_") || a.name.starts_with("pl_"))
        .filter_map(|a| {
            let len = f32::from(a.numframes) / a.framerate;
            let (first, last) = (a.delta_trans.first()?.1, a.delta_trans.last()?.1);
            let v = [last[0] - first[0], last[1] - first[1], last[2] - first[2]];
            let d = if v[0] + v[1] <= v[2] * 0.8 {
                (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
            } else {
                (v[0] * v[0] + v[1] * v[1]).sqrt()
            };
            (len > 0.0 && d > 1.0).then(|| (a.name.to_ascii_lowercase(), d / len))
        })
        .collect();
    scripts.t6 = Some(combat.scripts);
    loaded.scripts = scripts;

    loaded.weapons.absorb(combat.weapons);
    loaded.fpv_meshes.absorb(combat.fpv);
    loaded.world_weapons.absorb(combat.world_weapons);
    loaded.xanims.absorb_local(combat.xanims);
    loaded.tracers.absorb(combat.tracers);
    loaded.world.fx.absorb_missing(combat.fx);
    loaded.world.fx_models.absorb(combat.fx_models);
    if loaded.world.impact_fx.is_none() {
        loaded.world.impact_fx = combat.impact_fx;
    } else if combat.impact_fx.is_some() {
        // BO2's bullet impacts over the map game's own.
        loaded.world.impact_fx = combat.impact_fx;
    }
    if combat.pen_table.is_some() {
        loaded.pen_table = combat.pen_table;
    }
    // Sound: BO2's (guns, announcer, UI) with the map's ambience beside it.
    loaded.sound = Some(match (combat.sound, loaded.sound.take()) {
        (Ok(mut bo2), Some(Ok(map_sound))) => {
            bo2.absorb_missing_aliases_unresolved(map_sound);
            Ok(bo2)
        }
        (Ok(bo2), _) => Ok(bo2),
        (Err(error), Some(Ok(map_sound))) => {
            report.push(format!("bo2 rules: BO2 sound gap: {error}"));
            Ok(map_sound)
        }
        (Err(error), other) => other.unwrap_or(Err(error)),
    });
    report.push(format!("bo2 rules: {map} plays by Black Ops II's rules"));
    loaded.report.extend(report);
}

/// bo2zm: every sound alias a map script names as a whole string (the
/// banks' aliases that appear in a script's string table).
pub(super) fn script_sound_aliases(scripts: &crate::script_sources::T6ScriptSet) -> Vec<String> {
    let known: std::collections::HashSet<&str> =
        scripts.sound_aliases.iter().map(String::as_str).collect();
    let mut out: Vec<String> = scripts
        .objects
        .iter()
        .filter_map(|bytes| asset_t6::gsc::GscObject::parse(bytes).ok())
        .flat_map(|g| g.strings.into_values())
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| known.contains(s.as_str()))
        .collect();
    // Breakable props' break sounds (a mannequin's head popping off).
    for d in &scripts.destructibles {
        for p in &d.pieces {
            for st in &p.stages {
                if let Some(snd) = st.break_sound.as_ref().map(|s| s.to_ascii_lowercase())
                    && known.contains(snd.as_str())
                {
                    out.push(snd);
                }
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// A T6 vision file's saturation: the w of `vc_FSM "r g b saturation"` (its
/// Rec. 709 weights then the mix toward colour), 1 when it has none.
fn vision_saturation(bytes: &[u8]) -> f32 {
    vision_value(bytes, "vc_FSM").map_or(1.0, |v| v[3])
}

/// A T6 vision value `vc_<key> "x y z w"`.
fn vision_value(bytes: &[u8], key: &str) -> Option<[f32; 4]> {
    String::from_utf8_lossy(bytes).lines().find_map(|line| {
        let rest = line.trim().strip_prefix(key)?;
        if !rest.starts_with([' ', '\t']) {
            return None;
        }
        let mut v = [0.0f32; 4];
        let mut parts = rest.trim().trim_matches('"').split_whitespace();
        for x in &mut v {
            *x = parts.next()?.parse::<f32>().ok().filter(|f| f.is_finite())?;
        }
        Some(v)
    })
}

/// A T6 vision's colour grade as the game's colour table shader
/// (`hdr_create_lut2dv`) reads it, rows for `asset_world::T6WorldLighting::grade`:
/// the tone bands' slopes and offsets against the luma of the display value
/// (shadow falling over `vc_RS.x..vc_RE.x`, highlight rising over `.y`, mid
/// rising over `.z` and falling over `.w`), the red, green and blue rows of
/// the shadow, mid and highlight matrices (`vc_SM*`, `vc_MM*`, `vc_HM*`; w a
/// bias), then `vc_FGM` (gamma, as its reciprocal power), `vc_FSM` (luma weights, saturation) and
/// `vc_FBM` (how much of the grade is kept).
fn vision_grade(bytes: &[u8]) -> [[f32; 4]; 14] {
    let get = |key: &str, default: [f32; 4]| vision_value(bytes, key).unwrap_or(default);
    let rs = get("vc_RS", [0.0, 1.0, 0.0, 1.0]);
    let re = get("vc_RE", [1.0, 2.0, 1.0, 2.0]);
    let (mut slope, mut offset) = ([0.0f32; 4], [0.0f32; 4]);
    for i in 0..4 {
        let width = (re[i] - rs[i]).max(0.0001);
        // The shadow and the mid's fall go down, the others up.
        if i == 0 || i == 3 {
            slope[i] = -1.0 / width;
            offset[i] = re[i] / width;
        } else {
            slope[i] = 1.0 / width;
            offset[i] = -rs[i] / width;
        }
    }
    // The shader raises to cb131; an artist's gamma brightens above 1, so
    // the table takes its reciprocal (Die Rise's warm 1.05 / 0.995 / 0.957
    // as given turns its spawn blue, R/B 1.03 against the real 1.31).
    let fgm = get("vc_FGM", [1.0, 1.0, 1.0, 0.0]);
    let inv = |g: f32| if g > 0.0001 { 1.0 / g } else { 1.0 };
    let gamma = [inv(fgm[0]), inv(fgm[1]), inv(fgm[2]), fgm[3]];
    let row = |key: &str, i: usize| {
        let mut r = [0.0; 4];
        r[i] = 1.0;
        get(key, r)
    };
    [
        slope,
        offset,
        row("vc_SMR", 0),
        row("vc_MMR", 0),
        row("vc_HMR", 0),
        row("vc_SMG", 1),
        row("vc_MMG", 1),
        row("vc_HMG", 1),
        row("vc_SMB", 2),
        row("vc_MMB", 2),
        row("vc_HMB", 2),
        gamma,
        get("vc_FSM", [0.212585, 0.715195, 0.07222, 1.0]),
        get("vc_FBM", [1.0, 1.0, 1.0, 0.0]),
    ]
}
