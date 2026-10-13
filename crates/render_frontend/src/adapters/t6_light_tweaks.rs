//! bo2zm: the lighting dvars a Black Ops II map's client scripts tweak:
//! `r_lightTweakSunLight` (the sun's brightness; the sun colour draws times
//! it over the map's own), `r_sky_intensity_factor0` (the sky's brightness
//! toward the horizon) and `r_lightTweakSunColor`. The server sends the first
//! two as the client dvar `bo2zm_sunlight` ("sun factor0") and the colour as
//! `bo2zm_suncolor` ("r g b"); without them the map's own values draw.
//! The sky's brightness blends from factor0 to factor1 as the view tilts
//! from angle0 to angle1 (`skyColorMultiplier`). Buried's town turns them
//! from 13 and 1 to 18 and 10; Origins' snow pales the sun colour over ten
//! seconds (`lerp_suncolor_dvar`).

use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::assemble::drawsurf::command_context::t5_sky_intensity;
use crate::prepare::scene::camera::FpvLens;
use crate::prepare::scene::world::WorldScene;

pub(crate) fn register(app: &mut App) {
    app.add_systems(Update, apply_t6_light_tweaks.in_set(frame::RenderSet::Anim));
}

/// The scripts' sun colour lerp (`n_suncolor_lerp_time`).
const SUN_COLOR_LERP_SECS: f32 = 10.0;

/// The first `N` numbers of a dvar's text.
fn parse<const N: usize>(text: &str) -> Option<[f32; N]> {
    let mut it = text.split_whitespace().map(|x| x.parse::<f32>().ok());
    let mut v = [0.0; N];
    for x in &mut v {
        *x = it.next()??;
    }
    Some(v)
}

/// The sun colour drawn: lerping from `from` to `to`, `t` of the way.
#[derive(Default)]
struct SunColor {
    from: [f32; 3],
    to: [f32; 3],
    t: f32,
}

impl SunColor {
    fn now(&self) -> [f32; 3] {
        std::array::from_fn(|i| self.from[i] + (self.to[i] - self.from[i]) * self.t)
    }
}

/// A script dvar of the local player's, or the test aid env var before it.
fn dvar(
    presented: Option<&PresentedSnapshot>,
    local: Option<&LocalPresentClient>,
    env: &str,
    key: &str,
) -> Option<String> {
    std::env::var(env).ok().or_else(|| match (presented, local) {
        (Some(p), Some(l)) => p
            .snapshot()
            .and_then(|s| s.meta.script_dvars(l.0).string(key).map(str::to_owned)),
        _ => None,
    })
}

fn apply_t6_light_tweaks(
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    scene: Option<ResMut<WorldScene>>,
    cameras: Query<&GlobalTransform, With<FpvLens>>,
    time: Res<Time>,
    mut shown: Local<Option<([f32; 2], [f32; 3])>>,
    mut colour: Local<Option<SunColor>>,
) {
    let Some(mut scene) = scene else {
        *shown = None;
        *colour = None;
        return;
    };
    let Some(map) = scene.t6_light_tweaks else {
        if scene.t6_sun_sky_scale != [1.0; 4] {
            scene.t6_sun_sky_scale = [1.0; 4];
        }
        *shown = None;
        *colour = None;
        return;
    };
    let (presented, local) = (presented.as_deref(), local.as_deref());
    // IW4L_T6_SUNLIGHT="18 10", IW4L_T6_SUNCOLOR="r g b": as if the scripts
    // set them (test aids).
    let tweak = dvar(presented, local, "IW4L_T6_SUNLIGHT", "bo2zm_sunlight")
        .and_then(|v| parse::<2>(&v));
    let want = dvar(presented, local, "IW4L_T6_SUNCOLOR", "bo2zm_suncolor")
        .and_then(|v| parse::<3>(&v))
        .unwrap_or(map.sun_color);
    let said = (tweak.unwrap_or([map.sun_light, map.sky_intensity[2]]), want);
    if *shown != Some(said) {
        diag::info!(
            World,
            "bo2zm light tweaks: {tweak:?} colour {want:?} (map sun {} colour {:?} sky {:?})",
            map.sun_light,
            map.sun_color,
            map.sky_intensity
        );
        *shown = Some(said);
    }
    // The first colour snaps (the scripts' initial snap); a change lerps.
    let c = colour.get_or_insert_with(|| SunColor { from: want, to: want, t: 1.0 });
    if c.to != want {
        *c = SunColor { from: c.now(), to: want, t: 0.0 };
    }
    c.t = (c.t + time.delta_secs() / SUN_COLOR_LERP_SECS).min(1.0);
    let rgb = c.now();
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    // A map with no sky intensity set draws its sky as is.
    let mut sky = map.sky_intensity;
    if !(sky.iter().all(|v| v.is_finite()) && (sky[2] > 0.0 || sky[3] > 0.0)) {
        sky = [0.0, 0.0, 1.0, 1.0];
    }
    let mut sun = 1.0;
    if let Some([sun_light, factor0]) = tweak {
        sun = sun_light / map.sun_light;
        sky[2] = factor0;
    }
    let tint = |i: usize| {
        if map.sun_color[i] > 0.0001 { rgb[i] / map.sun_color[i] } else { 1.0 }
    };
    let scale = [
        sun * tint(0),
        sun * tint(1),
        sun * tint(2),
        t5_sky_intensity(sky, camera.forward().as_vec3().z),
    ];
    if scene.t6_sun_sky_scale != scale {
        scene.t6_sun_sky_scale = scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_the_first_numbers() {
        assert_eq!(parse::<2>("18 10"), Some([18.0, 10.0]));
        assert_eq!(parse::<3>("0.769943 0.894102 1"), Some([0.769943, 0.894102, 1.0]));
        assert_eq!(parse::<2>("18"), None);
        assert_eq!(parse::<2>(""), None);
    }

    #[test]
    fn sun_colour_lerps_from_where_it_was() {
        let c = SunColor { from: [0.0, 0.5, 1.0], to: [1.0, 0.5, 0.0], t: 0.25 };
        assert_eq!(c.now(), [0.25, 0.5, 0.75]);
    }
}
