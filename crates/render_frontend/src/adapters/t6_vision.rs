//! bo2zm: a Black Ops II script's vision. The server's visionset manager
//! (`maps/mp/_visionset_mgr.gsc`) names the vision a player sees (its
//! `visionset_slot` client field; Mob's Afterlife turns on `zm_afterlife`),
//! which the server sends as the client dvar `bo2zm_vision` ("name lerp",
//! lerp 0..1 from the map's own grade). The world's colour grade becomes
//! that vision's; with none (or one the zones lack) the map's own draws.
//! A third word fades from that vision instead of the map's (a player
//! down: BO2's `zombie_death` fades in from `zombie_last_stand`).
//!
//! Its first-person hands too: `setviewmodel` (Mob's Afterlife ghost hands)
//! arrives as the client dvar `bo2zm_viewhands`. And its overlay (Mob's
//! Afterlife screen, `zm_afterlife_filter`), the client dvar
//! `bo2zm_overlay` ("name lerp"): the map's filter material with that
//! name's middle word (`generic_filter_afterlife_alcatraz`).

use bevy::prelude::*;
use net::{LocalPresentClient, PresentedSnapshot};

use crate::prepare::scene::world::WorldScene;

pub(crate) fn register(app: &mut App) {
    app.init_resource::<frame::ScriptScreenFilter>().add_systems(
        Update,
        (apply_t6_vision, apply_t6_viewhands, apply_t6_overlay).in_set(frame::RenderSet::Anim),
    );
}

type Grade = [[f32; 4]; 14];

/// `bo2zm_vision`'s vision file, lerp and the vision file it fades from
/// ("zm_afterlife 1" -> `vision/zm_afterlife.vision`, 1.0, none;
/// "zombie_death 0.5 zombie_last_stand" fades from the last stand's); none
/// for an empty value.
pub fn t6_vision_wanted(value: &str) -> Option<(String, f32, Option<String>)> {
    let mut words = value.split_whitespace();
    let name = words.next()?.to_ascii_lowercase();
    let lerp = words
        .next()
        .and_then(|w| w.parse::<f32>().ok())
        .filter(|l| l.is_finite())
        .map_or(1.0, |l| l.clamp(0.0, 1.0));
    let from = words
        .next()
        .map(|w| format!("vision/{}.vision", w.to_ascii_lowercase()));
    Some((format!("vision/{name}.vision"), lerp, from))
}

fn mix(from: &Grade, to: &Grade, t: f32) -> Grade {
    let mut out = *from;
    for (row, (a, b)) in out.iter_mut().zip(from.iter().zip(to)) {
        for i in 0..4 {
            row[i] = a[i] + (b[i] - a[i]) * t;
        }
    }
    out
}

pub(crate) fn apply_t6_vision(
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    scene: Option<ResMut<WorldScene>>,
    mut shown: Local<Option<(usize, Option<(String, u32, Option<String>)>)>>,
) {
    let Some(mut scene) = scene else {
        *shown = None;
        return;
    };
    if scene.t6_script_grades.is_empty() {
        *shown = None;
        return;
    }
    let wanted = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => p.snapshot().and_then(|s| {
            s.meta
                .script_dvars(l.0)
                .string("bo2zm_vision")
                .and_then(t6_vision_wanted)
        }),
        _ => None,
    };
    // Keyed by the loaded grades too: a new map starts over.
    let key = (
        std::sync::Arc::as_ptr(&scene.t6_script_grades) as usize,
        wanted.as_ref().map(|(n, l, f)| (n.clone(), l.to_bits(), f.clone())),
    );
    if shown.as_ref() == Some(&key) {
        return;
    }
    let from = wanted
        .as_ref()
        .and_then(|(_, _, f)| f.as_ref())
        .and_then(|f| scene.t6_script_grades.iter().find(|(n, _)| n == f))
        .map(|(_, g)| *g);
    let base = from.or(scene.t6_vision_grade_loaded);
    let grade = match &wanted {
        Some((name, lerp, _)) => match scene.t6_script_grades.iter().find(|(n, _)| n == name) {
            Some((_, to)) => Some(match base {
                Some(from) => mix(&from, to, *lerp),
                None => *to,
            }),
            None => {
                diag::warn!(World, "bo2zm vision {name}: not in the map's zones");
                scene.t6_vision_grade_loaded
            }
        },
        None => scene.t6_vision_grade_loaded,
    };
    diag::info!(World, "bo2zm vision {wanted:?}");
    scene.t6_vision_grade = grade;
    *shown = Some(key);
}

pub(crate) fn apply_t6_viewhands(
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    hands: Option<ResMut<render_anim::FpvHandsOverride>>,
) {
    let Some(mut hands) = hands else {
        return;
    };
    let wanted = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => p.snapshot().and_then(|s| {
            s.meta
                .script_dvars(l.0)
                .string("bo2zm_viewhands")
                .map(|name| name.trim().to_ascii_lowercase())
                .filter(|name| !name.is_empty())
        }),
        _ => None,
    };
    if hands.0 != wanted {
        diag::info!(World, "bo2zm hands {wanted:?}");
        hands.0 = wanted;
    }
}

/// `bo2zm_overlay`'s filter word and lerp ("zm_afterlife_filter 1" ->
/// "afterlife", 1.0); none for an empty value.
pub fn t6_overlay_wanted(value: &str) -> Option<(String, f32)> {
    let mut words = value.split_whitespace();
    let name = words.next()?.to_ascii_lowercase();
    let lerp = words
        .next()
        .and_then(|w| w.parse::<f32>().ok())
        .filter(|l| l.is_finite())
        .map_or(1.0, |l| l.clamp(0.0, 1.0));
    let word = name.strip_prefix("zm_").unwrap_or(&name);
    let word = word.strip_suffix("_filter").unwrap_or(word);
    Some((word.to_owned(), lerp))
}

pub(crate) fn apply_t6_overlay(
    presented: Option<Res<PresentedSnapshot>>,
    local: Option<Res<LocalPresentClient>>,
    filters: Option<Res<assets::T6ScreenFilters>>,
    mut out: ResMut<frame::ScriptScreenFilter>,
) {
    let wanted = match (presented.as_deref(), local.as_deref()) {
        (Some(p), Some(l)) => p.snapshot().and_then(|s| {
            s.meta
                .script_dvars(l.0)
                .string("bo2zm_overlay")
                .and_then(t6_overlay_wanted)
        }),
        _ => None,
    };
    let next = match (&wanted, filters.as_deref()) {
        (Some((word, lerp)), Some(filters)) => {
            let material = filters
                .0
                .iter()
                .find(|f| f.material.strip_prefix("generic_filter_").is_some_and(|m| m.contains(word.as_str())))
                .map(|f| f.material.clone());
            frame::ScriptScreenFilter {
                amount: if material.is_some() { *lerp } else { 0.0 },
                material,
            }
        }
        _ => frame::ScriptScreenFilter::default(),
    };
    if *out != next {
        diag::info!(World, "bo2zm overlay {wanted:?} -> {:?} {}", next.material, next.amount);
        *out = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wanted_names_the_vision_file() {
        assert_eq!(
            t6_vision_wanted("zm_afterlife"),
            Some(("vision/zm_afterlife.vision".to_owned(), 1.0, None))
        );
        assert_eq!(
            t6_vision_wanted("ZM_Afterlife 0.5"),
            Some(("vision/zm_afterlife.vision".to_owned(), 0.5, None))
        );
        assert_eq!(t6_vision_wanted(""), None);
        assert_eq!(
            t6_vision_wanted("zombie_death 0.25 Zombie_Last_Stand"),
            Some((
                "vision/zombie_death.vision".to_owned(),
                0.25,
                Some("vision/zombie_last_stand.vision".to_owned())
            ))
        );
    }

    #[test]
    fn overlay_names_the_filter_word() {
        assert_eq!(
            t6_overlay_wanted("zm_afterlife_filter 0.5"),
            Some(("afterlife".to_owned(), 0.5))
        );
        assert_eq!(t6_overlay_wanted(" "), None);
    }
}
