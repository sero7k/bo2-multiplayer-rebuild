//! bo2mp: World at War maps in Black Ops II's look. A map's tiling
//! surfaces (concrete, brick, plaster, wood, metal sheeting, rubble, dirt,
//! sand, grass, snow, stone floors, roof tiles) take a Black Ops II
//! material of the same kind (colour, normal and specular maps from Black
//! Ops II's image packs), and the map is lit and graded as a Black Ops II
//! map of the same setting (its model): that map's colour grade, fog and
//! reflection probe, and its sun's hue on the map's own sun direction (the
//! map's baked lightmaps and shadows stay its own). Trim sheets, ornaments,
//! decals, windows, signs and foliage keep their own pictures: their
//! texture coordinates are laid out for those exact images.
//!
//! The swaps: first a map's own list (`RECIPES`), then the shared rules
//! (`RULES`, by the World at War colour image's name). The Black Ops II
//! materials come from a small palette of maps (Aftermath, Standoff,
//! Yemen, Castaway, Frostbite) plus the model. `BO2MP_BO2_LOOK=0` draws
//! every map as World at War drew it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use bevy::image::Image;

/// A map's model and its own swaps (its colour image -> a Black Ops II
/// material, `zone:material`; a bare material is the model's).
pub(crate) struct Recipe {
    pub map: &'static str,
    /// The Black Ops II map whose grade, fog, probe and sun it takes.
    pub model: &'static str,
    pub swaps: &'static [(&'static str, &'static str)],
}

/// Every World at War multiplayer map and the Black Ops II map of its
/// setting and mood (Zombies maps too): the burning Berlin ruins (Dome,
/// Downfall, Breach, Stalingrad, Nachtfeuer) as Nuketown Zombies' burnt
/// haze; the grim and the underground (Asylum, Subway) as Mob of the
/// Dead's. The rest: Berlin ruins as Aftermath; the German countryside and towns as
/// Standoff; snow as Frostbite; rain as Hydro; the night raids as Plaza;
/// the submarine pens as Cargo; the Pacific islands and jungles as
/// Castaway; Japanese castles and shrines as Magma.
pub(crate) const RECIPES: &[Recipe] = &[
    // Dome: the Reichstag in ruins, burning, under smoke. Nuketown Zombies'
    // light (its burnt orange grade, haze and dim sun) and its burnt
    // and ashen materials where Dome has the same kind of surface.
    Recipe {
        map: "mp_dome",
        model: "zm_nuked",
        swaps: &[
            ("berlin_wall_concrete_grungy_cracks_c", "zm_nuked:wpc/jun_art_concrete_base02_dark"),
            ("berlin_wall_concrete_holes_splats2_c", "mp_la:wpc/concrete_wall_broken"),
            ("berlin_wall_concrete_reichstag_int_c", "zm_nuked:wpc/concrete_exposed"),
            ("berlin_facade_concrete_lower_c", "zm_nuked:wpc/jun_art_concrete_base02_dark"),
            ("peleliu_wall_concrete_grimey_c", "mp_la:wpc/com_concrete_core_dusty"),
            ("berlin_floors_marble_tile_ornate_c", "mp_la:wpc/pb_marble_tile_floor"),
            ("okinawa_walls_wood_trenchboard1_c", "zm_nuked:wpc/zm_nuked_wood_planks_burnt"),
            ("okinawa_floor_wood_faded_boards_c", "zm_nuked:wpc/wood_burned"),
            ("berlin_floors_wood_plnks_dark1_c", "zm_nuked:wpc/zm_nuked_wood_planks_burnt_lambert"),
            ("berlin_terrain_rubble_large_c", "zm_nuked:wpc/zm_nuked_rubble_ash_dark"),
            ("berlin_terrain_rubble_small_c", "zm_nuked:wpc/rubble_ash"),
            ("berlin_roofs_brick_slate2_c", "zm_nuked:wpc/zm_nuked_roof_brick_slate_02"),
        ],
    },
    Recipe { map: "mp_downfall", model: "zm_nuked", swaps: &[] },
    Recipe { map: "mp_bgate", model: "zm_nuked", swaps: &[] },
    Recipe { map: "mp_stalingrad", model: "zm_nuked", swaps: &[] },
    Recipe { map: "mp_subway", model: "zm_prison", swaps: &[] },
    Recipe { map: "mp_seelow", model: "mp_village", swaps: &[] },
    Recipe { map: "mp_asylum", model: "zm_prison", swaps: &[] },
    Recipe { map: "mp_vodka", model: "mp_village", swaps: &[] },
    Recipe { map: "mp_outskirts", model: "mp_frostbite", swaps: &[] },
    Recipe { map: "mp_roundhouse", model: "mp_frostbite", swaps: &[] },
    Recipe { map: "mp_suburban", model: "mp_hydro", swaps: &[] },
    Recipe { map: "mp_nachtfeuer", model: "zm_nuked", swaps: &[] },
    Recipe { map: "mp_makin", model: "mp_nightclub", swaps: &[] },
    Recipe { map: "mp_docks", model: "mp_dockside", swaps: &[] },
    Recipe { map: "mp_makin_day", model: "mp_castaway", swaps: &[] },
    Recipe { map: "mp_airfield", model: "mp_castaway", swaps: &[] },
    Recipe { map: "mp_hangar", model: "mp_castaway", swaps: &[] },
    Recipe { map: "mp_drum", model: "mp_castaway", swaps: &[] },
    Recipe { map: "mp_kneedeep", model: "mp_castaway", swaps: &[] },
    Recipe { map: "mp_kwai", model: "mp_castaway", swaps: &[] },
    Recipe { map: "mp_castle", model: "mp_magma", swaps: &[] },
    Recipe { map: "mp_courtyard", model: "mp_magma", swaps: &[] },
    Recipe { map: "mp_shrine", model: "mp_magma", swaps: &[] },
];

/// Pictures laid out for one place (trim sheets, ornaments, decals,
/// windows, signs, foliage, effects) keep their own: never swapped.
const KEEP: &[&str] = &[
    "trim", "decal", "window", "sign", "ornate", "ornament", "column", "poster", "glass", "global_", "sky",
    "graffiti", "text", "frame", "door", "stair", "pipe", "wire", "cable", "grate", "mask", "foliage", "leaf",
    "leaves", "tuft", "atlas", "fence", "vine", "curtain", "banner", "flag", "cloth", "light", "lamp",
    "sandbag", "barbwire", "rail", "beam", "edge", "corner", "caulk", "clip", "water", "godray", "shadow",
    "test_", "case64", "rug", "wallpaper", "thatch", "wicker", "lacquer", "tatami", "copper", "chain", "rebar",
    "facade", "dome", "hdrl", "seams", "_end", "_top", "_bottom", "crater", "footprint", "trax",
    "tracks", "sulfur", "pools", "shore", "pillbx", "bank", "vodka_", "drum_", "sub_metal", "kwai_",
];

/// The shared swaps: the first rule whose every part the colour image's
/// name holds wins (`zone:material`).
const RULES: &[(&[&str], &str)] = &[
    // Snow and ground.
    (&["snow"], "mp_frostbite:wpc/dh_snow_powder_flat_01"),
    (&["rubble", "small"], "mp_la:wpc/rubble_concrete_dust01"),
    (&["rubble"], "mp_la:wpc/rubble_concrete_debris"),
    (&["asphalt"], "mp_la:wpc/asphalt_cracked_dark"),
    (&["terrain", "sand"], "mp_castaway:wpc/cas_sand_clean_crushed"),
    (&["coral"], "mp_castaway:wpc/cas_coral_floor"),
    (&["terrain", "grass"], "mp_village:wpc/intro_ground_grass_01"),
    (&["terrain", "mud"], "mp_village:wpc/mud_pak_sludge02_lambert"),
    (&["terrain", "dirt"], "mp_village:wpc/ch_ground_dirt03"),
    (&["terrain", "rock"], "mp_castaway:wpc/mtl_p6_cas_rock_large_01"),
    (&["terrain", "stone"], "mp_castaway:wpc/mtl_p6_cas_rock_medium_01"),
    (&["cobble"], "mp_village:wpc/stone_floor_cobblestone"),
    (&["wall", "rock"], "mp_socotra:wpc/ch_stonewall_02"),
    // Floors.
    (&["floor", "marble"], "mp_la:wpc/stone_floor_granite_tile01_cream"),
    (&["floor", "tile"], "mp_la:wpc/stone_floor_granite_tile01_cream"),
    (&["floor", "concrete"], "mp_la:wpc/concrete_floor_garage01"),
    (&["floor", "metal"], "mp_la:wpc/metal_floor_treadplate"),
    (&["floor", "wood"], "mp_village:wpc/mp_berlin_floor_wood_hardwood_02"),
    (&["floors", "wood"], "mp_village:wpc/mp_berlin_floor_wood_hardwood_02"),
    // Roofs.
    (&["roof", "ceramic"], "mp_village:wpc/ceramic_roof_tiles_moss"),
    // Walls.
    (&["brick", "white"], "mp_village:wpc/concrete_bricks_beige"),
    (&["brick", "painted"], "mp_village:wpc/concrete_bricks_beige"),
    (&["roofs", "brick"], "mp_village:wpc/ceramic_roof_tiles_moss"),
    (&["brick"], "mp_village:wpc/brick_wall_redbrick"),
    (&["plaster", "chipped"], "mp_village:wpc/paris_plaster_cracked_paint"),
    (&["plaster", "dirty"], "mp_village:wpc/paris_plaster_cracked_paint"),
    (&["plaster", "stone"], "mp_socotra:wpc/plaster_beige_crackwall01"),
    (&["ceiling", "plaster"], "mp_village:wpc/plaster_wall_plaster02_eggwhite"),
    (&["plaster"], "mp_village:wpc/plaster_wall_plaster01_eggwhite"),
    (&["concrete", "crack"], "mp_la:wpc/concrete_wall_broken"),
    (&["concrete", "holes"], "mp_la:wpc/concrete_wall_broken"),
    (&["concrete", "chip"], "mp_la:wpc/concrete_wall_broken"),
    (&["concrete", "charred"], "mp_la:wpc/concrete_wall_broken"),
    (&["concrete", "grim"], "mp_la:wpc/com_concrete_core_dusty"),
    (&["concrete", "grung"], "mp_la:wpc/concrete_wall_rough_base01"),
    (&["concrete"], "mp_la:wpc/concrete_wall_rough_base01"),
    (&["ceiling", "wood"], "mp_village:wpc/core_wood_dark"),
    (&["wood", "burnt"], "mp_village:wpc/core_wood_dark"),
    (&["wall", "wood"], "mp_la:wpc/mp_b_art_floor_wood_dirty_dark"),
    (&["walls", "wood"], "mp_la:wpc/mp_b_art_floor_wood_dirty_dark"),
    (&["wood", "planks"], "mp_la:wpc/mp_b_art_floor_wood_dirty_dark"),
    (&["wood", "boards"], "mp_la:wpc/mp_b_art_floor_wood_dirty_dark"),
    (&["bamboo"], "mp_socotra:wpc/wood_wall_bamboo"),
    (&["metal", "rust"], "mp_socotra:wpc/iw_ch_metal_rusty_02"),
    (&["corrug"], "mp_village:wpc/metal_wall_corrugated_raised"),
    (&["corrig"], "mp_village:wpc/metal_wall_corrugated_raised"),
    (&["metal", "corr"], "mp_village:wpc/metal_wall_corrugated_raised"),
    (&["metal", "siding"], "mp_village:wpc/metal_wall_corrugated_raised"),
    (&["metal", "painted"], "mp_socotra:wpc/jun_art_metal_paintchip_gray"),
    (&["metal", "panel"], "mp_socotra:wpc/jun_art_metal_paintchip_gray"),
    (&["metal", "burnt"], "mp_village:wpc/core_metal_dark"),
    (&["ceilings", "metal"], "mp_socotra:wpc/jun_art_metal_paintchip_gray"),
];

/// The recipe for a map, when it has one and the look is on.
pub(crate) fn recipe(map: &str) -> Option<&'static Recipe> {
    if std::env::var("BO2MP_BO2_LOOK").is_ok_and(|v| v.trim() == "0") {
        return None;
    }
    RECIPES.iter().find(|r| r.map.eq_ignore_ascii_case(map))
}

/// The Black Ops II material (`zone`, `material`) a World at War colour
/// image is drawn with on a map, if any.
pub(crate) fn swap_for(recipe: &Recipe, image: &str) -> Option<(&'static str, &'static str)> {
    let split = |target: &'static str| match target.split_once(':') {
        Some((zone, material)) => (zone, material),
        None => (recipe.model, target),
    };
    if let Some(&(_, target)) = recipe.swaps.iter().find(|(w, _)| w.eq_ignore_ascii_case(image)) {
        return Some(split(target));
    }
    let name = image.to_ascii_lowercase();
    if !name.ends_with("_c") || KEEP.iter().any(|k| name.contains(k)) {
        return None;
    }
    RULES
        .iter()
        .find(|(parts, _)| parts.iter().all(|p| name.contains(p)))
        .map(|&(_, target)| split(target))
}

/// One swapped surface's Black Ops II pictures.
pub(crate) struct Swap {
    pub material: &'static str,
    pub colour: Arc<Image>,
    pub normal: Option<Arc<Image>>,
    pub specular: Option<Arc<Image>>,
}

/// A map's look, read: its model map's zone (grade, fog, probes) and the
/// packs, and the swaps' pictures by World at War colour image.
pub(crate) struct Look {
    pub model: &'static str,
    pub capture: asset_t6::ZoneCapture,
    pub packs: asset_t6::PackSet,
    pub swaps: HashMap<String, Swap>,
}

/// Black Ops II's zone folder (where every model and palette map is).
fn zone_dir() -> Result<PathBuf, String> {
    let root = asset_transport::games_root_from_env().map_err(|e| e.to_string())?;
    let found = asset_transport::find_zone_file(&root, "t6:mp_la").map_err(|e| e.to_string())?;
    found.path.parent().map(Path::to_path_buf).ok_or_else(|| "no zone folder".to_owned())
}

/// Reads a map's model and the palette maps its swaps need, and decodes
/// each swap's pictures. `images`: the map's world colour images.
pub(crate) fn load(recipe: &'static Recipe, images: &[String], report: &mut Vec<String>) -> Option<Look> {
    let dir = match zone_dir() {
        Ok(d) => d,
        Err(e) => {
            report.push(format!("bo2 look: Black Ops II's maps not found ({e}); {} drawn as its own game drew it", recipe.map));
            return None;
        }
    };
    let packs = match asset_t6::PackSet::open_dir(&dir) {
        Ok(p) => p,
        Err(e) => {
            report.push(format!("bo2 look: image packs: {e}"));
            return None;
        }
    };
    let mut zones: HashMap<&'static str, Option<asset_t6::ZoneCapture>> = HashMap::new();
    let mut capture = |zone: &'static str, zones: &mut HashMap<&'static str, Option<asset_t6::ZoneCapture>>| {
        zones
            .entry(zone)
            .or_insert_with(|| asset_t6::capture_zone(&dir.join(format!("{zone}.ff"))).ok())
            .is_some()
    };
    if !capture(recipe.model, &mut zones) {
        report.push(format!("bo2 look: {} unreadable; {} drawn as its own game drew it", recipe.model, recipe.map));
        return None;
    }
    let mut swaps = HashMap::new();
    let mut decoded: HashMap<&'static str, Option<(Arc<Image>, Option<Arc<Image>>, Option<Arc<Image>>)>> =
        HashMap::new();
    let mut misses = Vec::new();
    for image in images {
        let Some((zone, material)) = swap_for(recipe, image) else { continue };
        if !capture(zone, &mut zones) {
            misses.push(format!("{zone} unreadable"));
            continue;
        }
        let pictures = decoded
            .entry(material)
            .or_insert_with(|| {
                let c = zones.get(zone).and_then(Option::as_ref)?;
                swap_images(c, &packs, material).map_err(|e| misses.push(format!("{material}: {e}"))).ok()
            })
            .clone();
        if let Some((colour, normal, specular)) = pictures {
            swaps.insert(image.clone(), Swap { material, colour, normal, specular });
        }
    }
    misses.sort();
    misses.dedup();
    report.push(format!(
        "bo2 look: {} in {}'s look: {} colour images swapped ({} Black Ops II materials from {} maps){}",
        recipe.map,
        recipe.model,
        swaps.len(),
        decoded.values().filter(|d| d.is_some()).count(),
        zones.values().filter(|z| z.is_some()).count(),
        if misses.is_empty() { String::new() } else { format!("; not: {}", misses.join("; ")) }
    ));
    let capture = zones.remove(recipe.model).flatten()?;
    Some(Look { model: recipe.model, capture, packs, swaps })
}

type Pictures = (Arc<Image>, Option<Arc<Image>>, Option<Arc<Image>>);

fn swap_images(capture: &asset_t6::ZoneCapture, packs: &asset_t6::PackSet, name: &str) -> Result<Pictures, String> {
    use super::t6_materials::{NORMAL_MAP_HASH, SPECULAR_MAP_HASH, colour_texture, decode};
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
    Ok((colour, normal, specular))
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

    fn dome() -> &'static Recipe {
        RECIPES.iter().find(|r| r.map == "mp_dome").unwrap()
    }

    #[test]
    fn every_world_at_war_map_has_a_model() {
        for map in [
            "mp_airfield", "mp_asylum", "mp_bgate", "mp_castle", "mp_courtyard", "mp_docks", "mp_dome",
            "mp_downfall", "mp_drum", "mp_hangar", "mp_kneedeep", "mp_kwai", "mp_makin", "mp_makin_day",
            "mp_nachtfeuer", "mp_outskirts", "mp_roundhouse", "mp_seelow", "mp_shrine", "mp_stalingrad",
            "mp_suburban", "mp_subway", "mp_vodka",
        ] {
            assert!(RECIPES.iter().any(|r| r.map == map), "{map}");
        }
    }

    #[test]
    fn rules_swap_tiling_surfaces_and_keep_trims() {
        let r = dome();
        assert_eq!(swap_for(r, "berlin_wall_brick_gritty_c"), Some(("mp_village", "wpc/brick_wall_redbrick")));
        assert_eq!(swap_for(r, "seelow_wall_brick_white01_c"), Some(("mp_village", "wpc/concrete_bricks_beige")));
        // Dome's own: Nuketown Zombies' ash and bare concrete.
        assert_eq!(swap_for(r, "berlin_terrain_rubble_small_c"), Some(("zm_nuked", "wpc/rubble_ash")));
        assert_eq!(swap_for(r, "berlin_wall_concrete_reichstag_int_c"), Some(("zm_nuked", "wpc/concrete_exposed")));
        assert_eq!(swap_for(r, "berlin_trim_concrete_ornate_group_c"), None);
        assert_eq!(swap_for(r, "berlin_window_warehouse_c"), None);
        assert_eq!(swap_for(r, "decal_cable_c"), None);
        assert_eq!(swap_for(r, "berlin_ceilings_metal_rust_c"), Some(("mp_socotra", "wpc/iw_ch_metal_rusty_02")));
        assert_eq!(swap_for(r, "makin_terrain_sand_dry_soft_c"), Some(("mp_castaway", "wpc/cas_sand_clean_crushed")));
    }
}
