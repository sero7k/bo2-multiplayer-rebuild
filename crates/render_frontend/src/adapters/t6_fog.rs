<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_frontend/src/adapters/t6_fog.rs
//! bo2zm M3: a Black Ops II world's fog bank as its client scripts pick it
//! (`setworldfogactivebank`, which the server sends as the client dvar
//! `bo2zm_fogbank`): the light table's two fog rows become that bank's.
//! Nuketown's game over picks bank 2 (no fog) for the rocket shot.
=======
//! bo2zm M3: a Black Ops II world's fog as the game picks it: the fog
//! volume the camera is in, among the banks the client scripts switched on
//! (`setworldfogactivebank`, a bit mask, which the server sends as the
//! client dvar `bo2zm_fogbank`; bank 1 until a script says otherwise).
//! Outside every such volume the map's initial fog draws. The light table's
//! two fog rows become the picked volume's. Nuketown's houses carry their
//! own (softer sun) fog; its game over picks bank 2 (no fog) for the rocket
//! shot. A fog modifier volume the camera is in (same banks) then scales
//! the fog: its half distance by the volume's depth scale, the fog and sun
//! fog colours and opacities by its colour adjust (BO2 blends into and out
//! of a modifier over its transition time; ours switches at once).
>>>>>>> /tmp/claude-0/zmhead/crates/render_frontend/src/adapters/t6_fog.rs

use std::sync::Arc;

use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::prepare::scene::camera::FpvLens;
use crate::prepare::scene::world::WorldScene;

/// The light table's fog rows (`assets::lane::t6`), after its 256 lights:
/// the sun fog blend, then the fog itself.
const SUN_FOG_SLOT: usize = 256;
const FOG_SLOT: usize = 257;

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, apply_t6_fog_bank.in_set(frame::RenderSet::Anim));
}

<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_frontend/src/adapters/t6_fog.rs
fn apply_t6_fog_bank(
=======
/// The fog volume at `pos` among `banks`: the first one holding it (inside
/// where `dot(n, p) + d <= 0` for every plane, as exposure volumes).
pub fn t6_fog_volume_at(
    volumes: &[asset_world::T6FogVolume],
    banks: u32,
    pos: [f32; 3],
) -> Option<usize> {
    volumes.iter().position(|v| {
        v.banks & banks != 0
            && !v.planes.is_empty()
            && v.planes
                .iter()
                .all(|p| p[0] * pos[0] + p[1] * pos[1] + p[2] * pos[2] + p[3] <= 0.0)
    })
}

/// The fog modifier volume at `pos` among `banks`: the first one holding it.
pub fn t6_fog_modifier_at(
    modifiers: &[asset_world::T6FogModifier],
    banks: u32,
    pos: [f32; 3],
) -> Option<usize> {
    modifiers.iter().position(|m| {
        m.banks & banks != 0
            && !m.planes.is_empty()
            && m.planes
                .iter()
                .all(|p| p[0] * pos[0] + p[1] * pos[1] + p[2] * pos[2] + p[3] <= 0.0)
    })
}

pub(crate) fn apply_t6_fog_bank(
>>>>>>> /tmp/claude-0/zmhead/crates/render_frontend/src/adapters/t6_fog.rs
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    scene: Option<ResMut<WorldScene>>,
    cameras: Query<&GlobalTransform, With<FpvLens>>,
    mut shown: Local<Option<(usize, u32, Option<usize>, Option<usize>)>>,
) {
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_frontend/src/adapters/t6_fog.rs
    let Some(mut scene) = scene else { return };
=======
    let Some(mut scene) = scene else {
        *shown = None;
        return;
    };
>>>>>>> /tmp/claude-0/zmhead/crates/render_frontend/src/adapters/t6_fog.rs
    if scene.t6_fog_banks.is_empty() {
        *shown = None;
        return;
    }
<<<<<<< /home/claude/bo2-multiplayer-rebuild/crates/render_frontend/src/adapters/t6_fog.rs
    let want = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => p
            .snapshot()
            .and_then(|s| {
                s.meta
                    .script_dvars(l.0)
                    .string("bo2zm_fogbank")
                    .and_then(|v| v.trim().parse::<u32>().ok())
            })
            .unwrap_or(0),
        _ => 0,
    };
    if want == scene.t6_fog_bank {
        return;
    }
    let mut table = (*scene.t6_lights_loaded).clone();
    let rows = scene
        .t6_fog_banks
        .iter()
        .find(|(bank, _)| *bank == want)
        .map(|(_, rows)| *rows);
    if let Some([sun, fog]) = rows
        && table.len() > FOG_SLOT
    {
        table[SUN_FOG_SLOT] = sun;
        table[FOG_SLOT] = fog;
    }
    diag::info!(World, "bo2zm fog bank {} -> {want}", scene.t6_fog_bank);
    scene.t6_lights = Arc::new(table);
    scene.t6_fog_bank = want;
=======
    // IW4L_T6_FOGBANK=4: as if a script picked that bank (test aid).
    let test = std::env::var("IW4L_T6_FOGBANK").ok().and_then(|v| v.trim().parse::<u32>().ok());
    let banks = match (presented.as_deref(), local.as_deref()) {
        _ if test.is_some() => test,
        (Some(p), Some(l)) => p.snapshot().and_then(|s| {
            s.meta
                .script_dvars(l.0)
                .string("bo2zm_fogbank")
                .and_then(|v| v.trim().parse::<u32>().ok())
        }),
        _ => None,
    }
    .filter(|&b| b != 0)
    .unwrap_or(1);
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let pos = camera.translation().to_array();
    let mut volume = t6_fog_volume_at(&scene.t6_fog_banks, banks, pos);
    // A script's bank switch (not the start bank) outside all its volumes
    // still takes the bank's first volume, as the switch did before volumes
    // were placed (Nuketown's game over).
    if volume.is_none() && banks != 1 {
        volume = scene.t6_fog_banks.iter().position(|v| v.banks & banks != 0);
    }
    let modifier = t6_fog_modifier_at(&scene.t6_fog_modifiers, banks, pos);
    // Keyed by the loaded table too: a new map starts over.
    let key = (
        Arc::as_ptr(&scene.t6_lights_loaded) as usize,
        banks,
        volume,
        modifier,
    );
    if *shown == Some(key) {
        return;
    }
    let loaded = Arc::clone(&scene.t6_lights_loaded);
    let mut table = (*scene.t6_lights).clone();
    let mut rows = match volume.and_then(|i| scene.t6_fog_banks.get(i)) {
        Some(v) => v.rows,
        None if loaded.len() > FOG_SLOT => [loaded[SUN_FOG_SLOT], loaded[FOG_SLOT]],
        None => return,
    };
    // The fog row: half distance at 1, opacity at 2, colour at 4..7, sun fog
    // colour at 8..11 and its opacity at 11 (`assets::lane::t6::fog_rows`).
    if let Some(m) = modifier.and_then(|i| scene.t6_fog_modifiers.get(i)) {
        let row = &mut rows[1];
        row[1] *= m.depth_scale;
        row[2] *= m.color_adjust[3];
        for k in 0..3 {
            row[4 + k] *= m.color_adjust[k];
            row[8 + k] *= m.color_adjust[k];
        }
        row[11] *= m.color_adjust[3];
    }
    if table.len() > FOG_SLOT {
        table[SUN_FOG_SLOT] = rows[0];
        table[FOG_SLOT] = rows[1];
        // `IW4L_T6_NO_FOG` draws without fog: a test aid for comparing.
        if std::env::var_os("IW4L_T6_NO_FOG").is_some() {
            table[FOG_SLOT][3] = 0.0;
        }
    }
    diag::info!(
        World,
        "bo2zm fog banks {banks:#x} volume {volume:?} modifier {modifier:?} at ({:.0} {:.0} {:.0})",
        pos[0],
        pos[1],
        pos[2]
    );
    scene.t6_lights = Arc::new(table);
    scene.t6_fog_bank = banks;
    *shown = Some(key);
>>>>>>> /tmp/claude-0/zmhead/crates/render_frontend/src/adapters/t6_fog.rs
}
