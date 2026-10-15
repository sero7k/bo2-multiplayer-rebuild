//! bo2mp: Black Ops III's look on a Black Ops II map (`BO2MP_BO3_LOOK=1`;
//! off by default, so Black Ops II plays as before). The world is drawn by
//! the fallback pass's Black Ops III branch (`t7_shade` in
//! `geometry_diagnostic.wgsl`, the one Black Ops III's own maps use,
//! `super::t7_light`): GGX sun highlights, the probe's reflection by
//! gloss, the sun shadowed by a sun map baked at load over the map's own
//! triangles (`fastfile_t7::light::SunMap`, the same bake), the sun's
//! shafts through it, and the filmic tone curve.
//!
//! What it takes from Black Ops II, as Black Ops II has them: the textures
//! (colour, normal, specular with gloss in its alpha: the forms the branch
//! reads), the sun (direction and colour), the sky cube, the reflection
//! probes (`rgb / a`), the fog, and the light without the sun: the
//! lightmap's ambient and directional pages for the world (a vertex colour
//! here is a tint or layer weights, not free for a baked light) and the
//! light grid for models. `t7_bo2` in the shader says which.
//!
//! `BO2MP_BO3_EXPOSURE` scales the exposure (default 2: Black Ops II's own
//! into the tone curve), `BO2MP_T7_HAZE` and `BO2MP_T7_SHAFTS` the shafts.

use asset_core::{T6Blend, T6Draw};
use asset_material::MaterialCatalog;
use asset_world::{ModelMesh, StaticModelPlacement, WorldDraw};
use fastfile_t7::bake::Caster;
use fastfile_t7::light::SunMap;

/// Whether Black Ops II maps take Black Ops III's look.
pub(super) fn on() -> bool {
    std::env::var("BO2MP_BO3_LOOK").is_ok_and(|v| v.trim() == "1")
}

/// How far, round the spawn points' square, the sun map reaches.
const MARGIN: f32 = 3000.0;

fn env_scale(name: &str, default: f32) -> f32 {
    std::env::var(name).ok().and_then(|v| v.trim().parse().ok()).filter(|v: &f32| v.is_finite() && *v >= 0.0).unwrap_or(default)
}

/// What a material does to the sun: none, partly (alpha tested), fully.
fn blocks(d: Option<&T6Draw>) -> Option<bool> {
    let d = d?;
    if d.unlit || d.effect || d.cloud {
        return None;
    }
    match d.blend {
        T6Blend::Opaque => Some(d.alpha_test),
        T6Blend::ShadowOnly => Some(false),
        _ => None,
    }
}

/// Turns Black Ops III's look on for a Black Ops II world: bakes the sun
/// map over the spawn points' square from the world's and placed models'
/// triangles and writes the branch's rows into the light table. `sun`:
/// toward the sun; `sun_colour`: its light. (The exposure is the map's as
/// it moves between its exposure volumes: the shader takes Black Ops II's.)
pub(super) fn apply(
    draw: &mut WorldDraw,
    catalog: &MaterialCatalog,
    meshes: &[ModelMesh],
    placements: &[Option<StaticModelPlacement>],
    spawns: &[[f32; 3]],
    sun: [f32; 3],
    sun_colour: [f32; 3],
    report: &mut Vec<String>,
) {
    let started = std::time::Instant::now();
    // The rows must be free (no primary light there).
    const ROWS: [usize; 4] = [super::t7_light::T7_FOG_ROW, super::t7_light::T7_FOG_ROW + 1, asset_world::T7_LIGHT_ROW, asset_world::T7_LIGHT_ROW + 1];
    if draw.t6_lights.len() < asset_world::T7_LIGHT_ROW + 2 {
        draw.t6_lights.resize(asset_world::T7_LIGHT_ROW + 2, [0.0; 16]);
    }
    if ROWS.iter().any(|&r| draw.t6_lights[r].iter().any(|v| *v != 0.0)) {
        report.push("bo3 look: off (the light table's rows 252..255 hold primary lights)".to_owned());
        return;
    }
    // The spawn points' square (else the world's bounds).
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in spawns {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    if lo[0] > hi[0] {
        lo = [draw.stats.min[0], draw.stats.min[1]];
        hi = [draw.stats.max[0], draw.stats.max[1]];
    }
    let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let span = ((hi[0] - lo[0]).max(hi[1] - lo[1]) + MARGIN).clamp(2048.0, 16000.0);
    let half = span * 0.5 + MARGIN;
    let inside = |p: [f32; 3]| (p[0] - centre[0]).abs() <= half && (p[1] - centre[1]).abs() <= half;
    let material_draw = |m: Option<usize>| m.and_then(|m| catalog.materials.get(m)).and_then(|m| m.t6_draw.as_ref());
    // The world's triangles.
    let mut casters: Vec<Caster> = Vec::new();
    for (s, &(start, count)) in draw.surface_index_ranges.iter().enumerate() {
        let Some(partial) = blocks(material_draw(draw.surface_materials.get(s).copied().flatten())) else { continue };
        let idx = &draw.packed_indices[start as usize..(start + count) as usize];
        for t in idx.chunks_exact(3) {
            let p = [0, 1, 2].map(|k| draw.positions.get(t[k] as usize).copied().unwrap_or_default());
            if p.iter().any(|v| inside(*v)) {
                casters.push(Caster { p, partial });
            }
        }
    }
    let world_tris = casters.len();
    // The placed models' (their first LOD), placed.
    for pl in placements.iter().flatten() {
        if !inside(pl.origin) {
            continue;
        }
        let Some(mesh) = meshes.get(pl.mesh) else { continue };
        for surf in mesh.surfaces() {
            let Some(partial) = blocks(material_draw(surf.material)) else { continue };
            let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(pos)) = surf.mesh.attribute(bevy::prelude::Mesh::ATTRIBUTE_POSITION) else { continue };
            let Some(indices) = surf.mesh.indices() else { continue };
            let world: Vec<[f32; 3]> = pos.iter().map(|v| pl.transform.transform_point(bevy::prelude::Vec3::from_array(*v)).to_array()).collect();
            let ix: Vec<usize> = indices.iter().collect();
            for t in ix.chunks_exact(3) {
                if let (Some(a), Some(b), Some(c)) = (world.get(t[0]), world.get(t[1]), world.get(t[2])) {
                    casters.push(Caster { p: [*a, *b, *c], partial });
                }
            }
        }
    }
    let z = spawns.iter().map(|p| p[2]).sum::<f32>() / spawns.len().max(1) as f32;
    let map = SunMap::render(&casters, sun, [centre[0], centre[1], z], span * 1.15, 4096);
    let s = map.depth.size;
    draw.t6_lut = Some(std::sync::Arc::new(asset_world::T6Lut { width: s, height: s, rgba: map.texels() }));
    // The branch's rows (`t7_light::Lit::light_rows`): exposure, the sun's
    // light over the sun colour drawn (1: Black Ops II's own), the sun
    // map's axes and packing, the test view, and the Black Ops II flag.
    let m = map.rows();
    let user = env_scale("BO2MP_BO3_EXPOSURE", 2.0).max(0.01);
    let mut a = [0f32; 16];
    a[0] = user;
    a[1] = 1.0;
    a[3] = 7.0;
    for k in 0..3 {
        a[4 + 4 * k..8 + 4 * k].copy_from_slice(&m[k]);
    }
    let mut b = [0f32; 16];
    b[..4].copy_from_slice(&m[3]);
    b[4] = super::t7_light::view();
    b[6] = 1.0;
    b[7] = 7.0;
    draw.t6_lights[asset_world::T7_LIGHT_ROW] = a;
    draw.t6_lights[asset_world::T7_LIGHT_ROW + 1] = b;
    // The shafts (`t7_fog_core` with its atmosphere off: Black Ops II's
    // own fog stays): the haze toward the sun, the sun's light times a
    // small density, shadowed by the sun map along the ray.
    let shafts = env_scale("BO2MP_T7_SHAFTS", 1.0);
    let haze = env_scale("BO2MP_T7_HAZE", 1.0) * 0.06;
    let hc = sun_colour.map(|c| c * haze);
    let fog: [[f32; 4]; 8] = [
        [0.0, 0.0, 0.0, 0.0],
        [hc[0], hc[1], hc[2], 1.0],
        [0.0, 0.0, 0.0, 65000.0],
        [sun[0], sun[1], sun[2], 0.6],
        [1.0, 1.0, 0.0, 0.0],
        [0.0; 4],
        [0.0; 4],
        [1.0 / 2000.0, 2500.0, if shafts > 0.0 { 16.0 } else { 0.0 }, shafts],
    ];
    for (i, r) in fog.iter().enumerate() {
        let row = super::t7_light::T7_FOG_ROW + i / 4;
        draw.t6_lights[row][4 * (i % 4)..4 * (i % 4) + 4].copy_from_slice(r);
    }
    draw.t7_light = true;
    report.push(format!(
        "bo3 look: on (BO2MP_BO3_LOOK=1): sun toward {sun:.3?} colour {sun_colour:.2?}, exposure x{:.2}, sun map {s}px over {:.0} units from {} triangles ({world_tris} world, {} placed; {} partial), {:.1}s",
        a[0],
        span * 1.15,
        casters.len(),
        casters.len() - world_tris,
        casters.iter().filter(|c| c.partial).count(),
        started.elapsed().as_secs_f32()
    ));
}
