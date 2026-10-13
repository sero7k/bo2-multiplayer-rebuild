//! bo2mp: a World at War map in Black Ops II's look. Its tiling surfaces
//! (concrete, brick, rubble, dirt, wooden and marble floors, asphalt) take
//! the Black Ops II material of the same kind (colour, normal and specular
//! maps from Black Ops II's image packs), and the map is lit and graded as
//! a Black Ops II map: that map's colour grade, fog and reflection probes,
//! its sun's colour on the map's own sun direction (the map's baked
//! lightmaps and shadows stay its own). Trim sheets, ornaments, decals and
//! windows keep their own pictures: their texture coordinates are laid out
//! for those exact images.
//!
//! One map so far: Dome (`mp_dome`, Berlin ruins) in Aftermath's look
//! (`mp_la`, a war-torn city in hazy daylight). `BO2MP_BO2_LOOK=0` draws it
//! as World at War drew it.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::image::Image;

/// A map's swaps: its colour image (as World at War names it) -> the Black
/// Ops II material drawn there instead.
pub(crate) struct Recipe {
    pub map: &'static str,
    /// The Black Ops II map whose materials, grade, fog and probes it takes.
    pub model: &'static str,
    pub swaps: &'static [(&'static str, &'static str)],
}

pub(crate) const RECIPES: &[Recipe] = &[Recipe {
    map: "mp_dome",
    model: "mp_la",
    swaps: &[
        // Concrete walls.
        ("berlin_wall_concrete_grungy_cracks_c", "wpc/concrete_wall_rough_base01"),
        ("berlin_wall_concrete_holes_splats2_c", "wpc/concrete_wall_broken"),
        ("berlin_wall_concrete_reichstag_int_c", "wpc/concrete_wall_pebbled01"),
        ("berlin_facade_concrete_lower_c", "wpc/concrete_wall_rough_base01"),
        ("peleliu_wall_concrete_grimey_c", "wpc/com_concrete_core_dusty"),
        ("peleliu_wall_concrete_cracked_c", "wpc/concrete_wall_broken"),
        // Brick.
        ("berlin_wall_brick_gritty_c", "wpc/ch_brick_wall_05"),
        ("seelow_wall_brick_dirty01_c", "wpc/ch_brick_wall_05"),
        ("seelow_wall_brick_white01_c", "wpc/ch_brick_wall_05"),
        ("berlin_wall_brick_bulletholes_c", "wpc/ch_brick_wall_05"),
        ("berlin_wall_brick_crumbly_c", "wpc/ch_brick_wall_05"),
        ("berlin_wall_brick_plain3_c", "wpc/ch_brick_wall_05"),
        // Rubble and ground.
        ("berlin_terrain_rubble_large_c", "wpc/rubble_concrete_debris"),
        ("berlin_terrain_rubble_small_c", "wpc/rubble_concrete_dust01"),
        ("peleliu_terrain_dirt_drymuddrk_c", "wpc/dirt_regular"),
        ("berlin_terrain_asphalt_old_c", "wpc/asphalt_cracked_dark"),
        // Wood.
        ("berlin_floors_wood_plnks_dark1_c", "wpc/mp_b_art_floor_wood_dirty_dark"),
        ("okinawa_walls_wood_trenchboard1_c", "wpc/mp_b_art_floor_wood_dirty_dark"),
        ("okinawa_floor_wood_faded_boards_c", "wpc/wood_hardwoodfloor_01"),
        // Stone floors.
        ("berlin_floors_marble_tile_ornate_c", "wpc/pb_marble_tile_floor"),
        ("berlin_floors_marble_tiles_grimy_c", "wpc/stone_floor_granite_tile01_cream"),
    ],
}];

/// The recipe for a map, when it has one and the look is on.
pub(crate) fn recipe(map: &str) -> Option<&'static Recipe> {
    if std::env::var("BO2MP_BO2_LOOK").is_ok_and(|v| v.trim() == "0") {
        return None;
    }
    RECIPES.iter().find(|r| r.map.eq_ignore_ascii_case(map))
}

/// One swapped surface's Black Ops II pictures.
pub(crate) struct Swap {
    pub material: &'static str,
    pub colour: Arc<Image>,
    pub normal: Option<Arc<Image>>,
    pub specular: Option<Arc<Image>>,
}

/// A recipe's model map, read: its zone (grade, fog, probes) and packs,
/// and the swaps' pictures by World at War colour image.
pub(crate) struct Look {
    pub model: &'static str,
    pub capture: asset_t6::ZoneCapture,
    pub packs: asset_t6::PackSet,
    pub swaps: HashMap<&'static str, Swap>,
}

/// The model map's zone file, in Black Ops II's zone folder.
fn model_zone(model: &str) -> Result<PathBuf, String> {
    let root = asset_transport::games_root_from_env().map_err(|e| e.to_string())?;
    asset_transport::find_zone_file(&root, &format!("t6:{model}"))
        .map(|z| z.path)
        .map_err(|e| e.to_string())
}

/// Reads a recipe's model map and decodes every swap's pictures.
pub(crate) fn load(recipe: &'static Recipe, report: &mut Vec<String>) -> Option<Look> {
    let path = match model_zone(recipe.model) {
        Ok(p) => p,
        Err(e) => {
            report.push(format!("bo2 look: {} not found ({e}); {} drawn as its own game drew it", recipe.model, recipe.map));
            return None;
        }
    };
    let capture = match asset_t6::capture_zone(&path) {
        Ok(c) => c,
        Err(e) => {
            report.push(format!("bo2 look: {} unreadable ({e})", path.display()));
            return None;
        }
    };
    let dir = path.parent()?.to_path_buf();
    let packs = match asset_t6::PackSet::open_dir(&dir) {
        Ok(p) => p,
        Err(e) => {
            report.push(format!("bo2 look: image packs: {e}"));
            return None;
        }
    };
    let mut swaps = HashMap::new();
    let mut misses = Vec::new();
    for &(waw, bo2) in recipe.swaps {
        match swap_images(&capture, &packs, bo2) {
            Ok(s) => {
                swaps.insert(waw, s);
            }
            Err(e) => misses.push(format!("{bo2}: {e}")),
        }
    }
    report.push(format!(
        "bo2 look: {} in {}'s look: {} of {} swaps decoded{}",
        recipe.map,
        recipe.model,
        swaps.len(),
        recipe.swaps.len(),
        if misses.is_empty() { String::new() } else { format!(" (not: {})", misses.join("; ")) }
    ));
    Some(Look { model: recipe.model, capture, packs, swaps })
}

fn swap_images(capture: &asset_t6::ZoneCapture, packs: &asset_t6::PackSet, name: &'static str) -> Result<Swap, String> {
    use super::t6_materials::{colour_texture, decode, NORMAL_MAP_HASH, SPECULAR_MAP_HASH};
    let material = capture
        .materials
        .iter()
        .find(|m| m.name == name)
        .ok_or_else(|| "not in the zone".to_owned())?;
    let image_of = |hash: u32| {
        material
            .textures
            .iter()
            .find(|t| t.name_hash == hash)
            .and_then(|t| t.image)
            .and_then(|k| capture.images.get(k.index))
            .filter(|img| img.streamed_parts > 0)
    };
    let colour = colour_texture(material, capture)
        .and_then(|t| t.image)
        .and_then(|k| capture.images.get(k.index))
        .ok_or_else(|| "no colour map".to_owned())?;
    let colour = Arc::new(decode(packs, colour)?);
    let normal = image_of(NORMAL_MAP_HASH).and_then(|i| decode(packs, i).ok()).map(Arc::new);
    let specular = image_of(SPECULAR_MAP_HASH).and_then(|i| decode(packs, i).ok()).map(Arc::new);
    Ok(Swap { material: name, colour, normal, specular })
}

/// The model map's colour grade, made while the world loads and installed
/// after the Black Ops II rules' own (which would put the map's own film
/// vision back).
static PENDING_GRADE: Mutex<Option<asset_world::T6Grade>> = Mutex::new(None);

pub(crate) fn set_pending_grade(grade: Option<asset_world::T6Grade>) {
    if let Ok(mut g) = PENDING_GRADE.lock() {
        *g = grade;
    }
}

pub(crate) fn take_pending_grade() -> Option<asset_world::T6Grade> {
    PENDING_GRADE.lock().ok().and_then(|mut g| g.take())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_dome_has_a_recipe_and_swaps_are_colour_maps() {
        assert!(RECIPES.iter().all(|r| r.swaps.iter().all(|(w, b)| w.ends_with("_c") && b.starts_with("wpc/"))));
        assert_eq!(RECIPES.len(), 1);
        assert_eq!(RECIPES[0].map, "mp_dome");
    }
}
