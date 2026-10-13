//! bo2zm: a Black Ops II world's exposure as the game picks it: from the
//! exposure volume the camera is in (`GfxWorld::exposureVolumes`), feathered
//! at its edges, and eased toward over time by the volume's luminance
//! increase / decrease scales. Every map but Nuketown has several volumes;
//! taking only the first one drew Origins 10-20x too bright.

use crate::prepare::scene::camera::FpvLens;
use bevy::prelude::*;

use crate::assemble::drawsurf::MapT5SunParseExposure;
use crate::prepare::scene::world::WorldScene;

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, apply_t6_exposure.in_set(frame::RenderSet::Anim));
}

/// Stops per second the exposure eases at, times the volume's scale.
const EASE_STOPS_PER_SECOND: f32 = 2.0;

/// The exposure (stops) at `pos` and the volume it came from: the first
/// volume holding it, blended over its feather range toward `outside`.
pub fn t6_exposure_at(
    volumes: &[asset_world::T6ExposureVolume],
    pos: [f32; 3],
    outside: f32,
) -> (f32, Option<usize>) {
    // Inside a volume is where every plane's `dot(normal, pos) + dist` is
    // <= 0: read that way Nuketown's three volumes are its two houses (and
    // the yellow house's garage), read the other way they sit mirrored in
    // the street. IW4L_T6_EXPOSURE_FLIP=1 takes the other side (test aid).
    let flip = if std::env::var_os("IW4L_T6_EXPOSURE_FLIP").is_some() {
        1.0
    } else {
        -1.0
    };
    for (i, v) in volumes.iter().enumerate() {
        if v.planes.is_empty() {
            continue;
        }
        // Distance inside the hull: the nearest plane.
        let depth = v
            .planes
            .iter()
            .map(|p| (p[0] * pos[0] + p[1] * pos[1] + p[2] * pos[2]) * flip - p[3])
            .fold(f32::INFINITY, f32::min);
        if depth < 0.0 {
            continue;
        }
        let w = if v.feather_range > 0.0 {
            (depth / v.feather_range).min(1.0)
        } else {
            1.0
        };
        return (outside + (v.exposure - outside) * w, Some(i));
    }
    (outside, None)
}

#[derive(Default)]
struct ExposureState {
    /// The stops shown, the map's start exposure, and the last volume.
    current: Option<f32>,
    start: f32,
    volume: Option<Option<usize>>,
}

fn apply_t6_exposure(
    time: Res<Time>,
    scene: Option<Res<WorldScene>>,
    exposure: Option<ResMut<MapT5SunParseExposure>>,
    cameras: Query<&GlobalTransform, With<FpvLens>>,
    mut state: Local<ExposureState>,
) {
    let (Some(scene), Some(mut exposure)) = (scene, exposure) else {
        *state = ExposureState::default();
        return;
    };
    // IW4L_T6_EXPOSURE=<stops>: that exposure everywhere (test aid).
    if let Some(stops) = std::env::var("IW4L_T6_EXPOSURE")
        .ok()
        .and_then(|v| v.trim().parse::<f32>().ok())
    {
        let shown = (-stops).exp2();
        if exposure.exposure != shown {
            exposure.exposure = shown;
        }
        return;
    }
    // IW4L_T6_EXPOSURE_VOLUMES=0: the map's sun exposure everywhere (test
    // aid).
    if scene.t6_exposure_volumes.is_empty()
        || std::env::var("IW4L_T6_EXPOSURE_VOLUMES").is_ok_and(|v| v == "0")
    {
        return;
    }
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let current = match state.current {
        Some(c) => c,
        None => {
            // The scale the map loaded with (its sun's, outside the volumes).
            let start = -exposure.exposure.max(1e-6).log2();
            state.start = start;
            state.current = Some(start);
            start
        }
    };
    let pos = camera.translation().to_array();
    let (target, volume) = t6_exposure_at(&scene.t6_exposure_volumes, pos, state.start);
    if state.volume != Some(volume) {
        diag::info!(
            World,
            "bo2zm exposure volume {:?} at ({:.0} {:.0} {:.0}): {target:.2} stops",
            volume,
            pos[0],
            pos[1],
            pos[2]
        );
        state.volume = Some(volume);
    }
    // Brighter scenes (a smaller exposure) ease by the increase scale.
    let scale = volume
        .and_then(|i| scene.t6_exposure_volumes.get(i))
        .map_or(1.0, |v| {
            if target < current {
                v.increase_scale
            } else {
                v.decrease_scale
            }
        })
        .max(0.05);
    let step = EASE_STOPS_PER_SECOND * scale * time.delta_secs();
    let next = if (target - current).abs() <= step {
        target
    } else {
        current + step * (target - current).signum()
    };
    state.current = Some(next);
    // Shown in 1/16 stop steps: the props are rebuilt on each change.
    let shown = (-(next * 16.0).round() / 16.0).exp2();
    if shown != exposure.exposure {
        exposure.exposure = shown;
    }
}
