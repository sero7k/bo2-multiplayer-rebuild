//! bo2zm: a Black Ops II world's colour table (`GfxWorld::lutMaterial`) as
//! the game picks it each frame: the table of the first colour table volume
//! the camera stands in (`GfxWorld::lutVolumes`), table 0 outside them all,
//! or the one a client script forces (`setlutscriptindex`). Each table keeps
//! a weight that rises toward the current table at the volume's transition
//! rate; the two last tables are drawn blended by those weights.

use crate::prepare::scene::camera::FpvLens;
use bevy::prelude::*;

use crate::prepare::scene::world::WorldScene;

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, apply_t6_lut.in_set(frame::RenderSet::Anim));
}

/// Tables a world can hold (`lutIndex` is five bits).
const TABLES: usize = 32;
/// Weight per millisecond when a volume names no transition time.
const INSTANT_RATE: f32 = 0.256;

/// The first volume holding `pos` in bank `banks`: inside is where every
/// plane's `dot(normal, pos) + dist` is <= 0 (as for exposure volumes).
pub fn t6_lut_volume_at(
    volumes: &[asset_world::T6LutVolume],
    pos: [f32; 3],
    banks: u32,
) -> Option<usize> {
    volumes.iter().position(|v| {
        v.banks & banks != 0
            && !v.planes.is_empty()
            && v.planes
                .iter()
                .all(|p| p[0] * pos[0] + p[1] * pos[1] + p[2] * pos[2] + p[3] <= 0.0)
    })
}

/// Weight per millisecond for a volume's `lutTransitionTime`: none (0) is
/// at once; under 1 it is read as a rate already, else as seconds.
fn rate(time: f32) -> f32 {
    if !(time > 0.0) {
        INSTANT_RATE
    } else if time < 1.0 {
        time
    } else {
        0.001 / time
    }
}

struct LutState {
    weights: [f32; TABLES],
    /// The tables drawn (previous, current) and the volume behind each.
    prev: (u32, Option<usize>),
    cur: (u32, Option<usize>),
    started: bool,
}

impl Default for LutState {
    fn default() -> Self {
        Self {
            weights: [0.0; TABLES],
            prev: (0, None),
            cur: (0, None),
            started: false,
        }
    }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn apply_t6_lut(
    time: Res<Time>,
    scene: Option<ResMut<WorldScene>>,
    cameras: Query<&GlobalTransform, With<FpvLens>>,
    mut state: Local<LutState>,
) {
    let Some(mut scene) = scene else {
        *state = LutState::default();
        return;
    };
    // IW4L_T6_NO_LUT=1: no colour table (test aid).
    // (bo2mp: a Black Ops III world's `t6_lut` is its sun map.)
    let Some(lut) = scene
        .t6_lut
        .clone()
        .filter(|_| !scene.t7_light && std::env::var_os("IW4L_T6_NO_LUT").is_none())
    else {
        if scene.t6_lut_pick[3] != 0.0 {
            scene.t6_lut_pick = [0.0, 0.0, 1.0, 0.0];
        }
        *state = LutState::default();
        return;
    };
    let tables = (lut.height / 32).max(1);
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let pos = camera.translation().to_array();
    let volume = t6_lut_volume_at(&scene.t6_lut_volumes, pos, 1);
    let table = scene
        .t6_lut_script
        .or_else(|| volume.and_then(|i| scene.t6_lut_volumes.get(i)).map(|v| v.table))
        .unwrap_or(0)
        .min(tables - 1);
    if !state.started {
        state.started = true;
        state.weights = [0.0; TABLES];
        state.weights[table as usize] = 1.0;
        state.prev = (table, volume);
        state.cur = (table, volume);
    } else if state.cur.0 != table {
        diag::info!(
            World,
            "bo2zm colour table {table} (volume {volume:?}) at ({:.0} {:.0} {:.0})",
            pos[0],
            pos[1],
            pos[2]
        );
        state.prev = state.cur;
        state.cur = (table, volume);
    } else {
        state.cur.1 = volume;
    }
    let volume_time = |v: Option<usize>| {
        v.and_then(|i| scene.t6_lut_volumes.get(i))
            .map(|v| v.transition_time)
    };
    let step = time.delta_secs()
        * 1000.0
        * rate(
            volume_time(state.cur.1)
                .or_else(|| volume_time(state.prev.1))
                .unwrap_or(0.0),
        );
    state.weights[table as usize] += 2.0 * step;
    for w in state.weights.iter_mut() {
        *w = (*w - step).clamp(0.0, 1.0);
    }
    let (a, b) = (state.prev.0, state.cur.0);
    let (wa, wb) = (state.weights[a as usize], state.weights[b as usize]);
    let w = if a == b || wa + wb <= 0.0 {
        0.0
    } else {
        smoothstep(wa / (wa + wb))
    };
    let pick = [(a * 32) as f32, (b * 32) as f32, w, 1.0];
    if scene.t6_lut_pick != pick {
        scene.t6_lut_pick = pick;
    }
}
