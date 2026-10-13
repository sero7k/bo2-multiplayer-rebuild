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
    /// The model map's sky (its `*_skycubemaphdr_*` material's cube, Black
    /// Ops II's own encoding), drawn in place of the map's own: World at
    /// War's skies are small cubes and looked poor under Black Ops II's
    /// grade. `None`: the model has no readable sky; the map keeps its own.
    pub sky: Option<(String, Arc<Image>)>,
}

/// The model map's sky cube: (material name, decoded cube).
fn model_sky(capture: &asset_t6::ZoneCapture, packs: &asset_t6::PackSet) -> Result<(String, Arc<Image>), String> {
    let material = capture
        .materials
        .iter()
        .find(|m| {
            m.technique_set
                .and_then(|k| capture.technique_sets.get(k.index))
                .is_some_and(|t| t.name.contains("skycubemap"))
        })
        .ok_or_else(|| "no sky material".to_owned())?;
    let image = super::t6_materials::colour_texture(material, capture)
        .and_then(|t| t.image)
        .and_then(|k| capture.images.get(k.index))
        .ok_or_else(|| format!("{}: no colour map", material.name))?;
    let cube = super::t6_materials::decode(packs, image).map_err(|e| format!("{}: {e}", material.name))?;
    Ok((material.name.clone(), Arc::new(cube)))
}

/// bo2mp: maps whose borrowed sky is pulled toward a colour (by how much,
/// 0..1): Dome burns, so its sky is the orange of the fires and smoke.
const SKY_TINTS: &[(&str, [f32; 3], f32)] = &[("mp_dome", [1.0, 0.52, 0.22], 0.7)];

/// The sky's colour pulled toward `colour` at its own brightness (each
/// face's top level, its HDR scale in alpha kept); one level, RGBA8.
fn tinted_sky(sky: &Image, colour: [f32; 3], amount: f32) -> Option<Image> {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let fmt = sky.texture_descriptor.format;
    let (block, decode): (usize, fn(&[u8], &mut [u8], usize)) = match fmt {
        TextureFormat::Bc1RgbaUnorm | TextureFormat::Bc1RgbaUnormSrgb => (8, bcdec_rs::bc1),
        TextureFormat::Bc2RgbaUnorm | TextureFormat::Bc2RgbaUnormSrgb => (16, bcdec_rs::bc2),
        TextureFormat::Bc3RgbaUnorm | TextureFormat::Bc3RgbaUnormSrgb => (16, bcdec_rs::bc3),
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => (0, |_, _, _| {}),
        _ => return None,
    };
    let size = sky.texture_descriptor.size;
    let (w, h, faces) = (size.width as usize, size.height as usize, size.depth_or_array_layers as usize);
    let levels = sky.texture_descriptor.mip_level_count.max(1) as usize;
    let data = sky.data.as_ref()?;
    let level_bytes = |l: usize| {
        let (lw, lh) = ((w >> l).max(1), (h >> l).max(1));
        if block == 0 { lw * lh * 4 } else { lw.div_ceil(4) * lh.div_ceil(4) * block }
    };
    let face_bytes: usize = (0..levels).map(level_bytes).sum();
    let lum_of = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let k = 1.0 / lum_of(colour).max(0.05);
    let mut out = Vec::with_capacity(w * h * 4 * faces);
    let mut tile = [0u8; 64];
    for face in 0..faces {
        let top = data.get(face * face_bytes..face * face_bytes + level_bytes(0))?;
        let mut rgba = vec![0u8; w * h * 4];
        if block == 0 {
            rgba.copy_from_slice(top);
        } else {
            let (bw, bh) = (w.div_ceil(4), h.div_ceil(4));
            for by in 0..bh {
                for bx in 0..bw {
                    let at = (by * bw + bx) * block;
                    decode(&top[at..at + block], &mut tile, 16);
                    for y in 0..4 {
                        for x in 0..4 {
                            let (px, py) = (bx * 4 + x, by * 4 + y);
                            if px < w && py < h {
                                let d = (py * w + px) * 4;
                                rgba[d..d + 4].copy_from_slice(&tile[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4]);
                            }
                        }
                    }
                }
            }
        }
        for p in rgba.chunks_exact_mut(4) {
            let c = [f32::from(p[0]) / 255.0, f32::from(p[1]) / 255.0, f32::from(p[2]) / 255.0];
            let lum = lum_of(c);
            for i in 0..3 {
                let v = c[i] * (1.0 - amount) + lum * colour[i] * k * amount;
                p[i] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
        out.extend_from_slice(&rgba);
    }
    let format = if fmt.is_srgb() { TextureFormat::Rgba8UnormSrgb } else { TextureFormat::Rgba8Unorm };
    let mut image = Image::new(
        Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: faces as u32 },
        TextureDimension::D2,
        out,
        format,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_view_descriptor = sky.texture_view_descriptor.clone();
    image.sampler = sky.sampler.clone();
    Some(image)
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
    let sky = match model_sky(&capture, &packs) {
        Ok(found) => {
            report.push(format!("bo2 look: sky from {}: {}", recipe.model, found.0));
            match SKY_TINTS.iter().find(|t| t.0 == recipe.map) {
                Some(&(_, colour, amount)) => match tinted_sky(&found.1, colour, amount) {
                    Some(t) => {
                        report.push(format!("bo2 look: sky tinted {colour:?} x{amount}"));
                        Some((found.0, Arc::new(t)))
                    }
                    None => {
                        report.push("bo2 look: sky tint skipped (format)".to_owned());
                        Some(found)
                    }
                },
                None => Some(found),
            }
        }
        Err(e) => {
            report.push(format!("bo2 look: {}'s sky not used ({e}); the map keeps its own", recipe.model));
            None
        }
    };
    Some(Look { model: recipe.model, capture, packs, swaps, sky })
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
/// bo2mp: a World at War map's own effects (its createfx: fires, smoke
/// columns, battle smoke) drawn as Black Ops II effects of the same kind
/// from its model map, when the model is Nuketown Zombies (the burning
/// Berlin maps: Dome, Downfall, ...). By the World at War effect's file
/// name, first match; effects with no match (god rays, mist) are left out.
pub(crate) const NUKED_FX_SWAPS: &[(&str, &str)] = &[
    ("fire_small_detail", "fire/fx_fire_xsm_no_flicker"),
    ("fire_tree_trunk", "fire/fx_nic_fire_pole_md_long"),
    ("fire_large", "maps/zombie/fx_zmb_tranzit_fire_lrg"),
    ("fire_lg", "maps/zombie/fx_zmb_tranzit_fire_lrg"),
    ("fire_medium", "maps/zombie/fx_zmb_nuke_fire_med"),
    ("fire_med", "maps/zombie/fx_zmb_nuke_fire_med"),
    ("fire_small", "fire/fx_nic_fire_line_sm"),
    ("fire_sm", "fire/fx_nic_fire_line_sm"),
    ("smoke_column_tall", "smoke/fx_smk_fire_lg_black"),
    ("smoke_column", "smoke/fx_smk_plume_md_blk_wispy_dist"),
    ("battlesmoke", "smoke/fx_smk_linger_lit_slow"),
    ("embers", "env/fire/fx_embers_falling_md"),
    ("ash", "maps/zombie/fx_zmb_nuke_burning_ash_gusty"),
];

/// The map's effects as Black Ops II's (`fxid` is the Black Ops II
/// effect's name), from its createfx and `level._effect` scripts
/// (`text(name)`: a raw file of its zone), and the zone they come from.
pub(crate) fn map_fx(map: &str, text: impl Fn(&str) -> Option<String>, report: &mut Vec<String>) -> Option<(Vec<asset_audio::CreateFxOneshot>, &'static str)> {
    let recipe = recipe(map)?;
    if recipe.model != "zm_nuked" {
        return None;
    }
    let lower = map.to_ascii_lowercase();
    let placed = [
        format!("maps/mp/createfx/{lower}_fx.gsc"),
        format!("clientscripts/mp/createfx/{lower}_fx.csc"),
        format!("maps/createfx/{lower}_fx.gsc"),
    ]
    .iter()
    .find_map(|n| text(n))?;
    let mut shots = asset_audio::parse_createfx_effects_and_loops(&placed);
    if let Some(src) = [format!("maps/mp/{lower}_fx.gsc"), format!("clientscripts/mp/{lower}_fx.csc")].iter().find_map(|n| text(n)) {
        let aliases = asset_audio::parse_createfx_effect_aliases(&src);
        asset_audio::apply_createfx_effect_aliases(&mut shots, &aliases);
    }
    let total = shots.len();
    let mut kinds: std::collections::BTreeMap<&'static str, usize> = std::collections::BTreeMap::new();
    shots.retain_mut(|s| {
        let id = s.fxid.to_ascii_lowercase();
        match NUKED_FX_SWAPS.iter().find(|(k, _)| id.contains(k)) {
            Some((_, to)) => {
                *kinds.entry(to).or_default() += 1;
                s.fxid = (*to).to_owned();
                // (A loop effect's delay is its repeat; these loop themselves.)
                s.delay = s.delay.min(0.0);
                true
            }
            None => false,
        }
    });
    report.push(format!("bo2 look: {} of {total} map effects drawn as {}'s: {kinds:?}", shots.len(), recipe.model));
    Some((shots, recipe.model))
}

static PENDING_FX: Mutex<Option<(Vec<asset_audio::CreateFxOneshot>, &'static str)>> = Mutex::new(None);

pub(crate) fn set_pending_fx(fx: Option<(Vec<asset_audio::CreateFxOneshot>, &'static str)>) {
    if let Ok(mut g) = PENDING_FX.lock() {
        *g = fx;
    }
}

pub(crate) fn take_pending_fx() -> Option<(Vec<asset_audio::CreateFxOneshot>, &'static str)> {
    PENDING_FX.lock().ok().and_then(|mut g| g.take())
}

/// The model zone with only the named effects (and the effects they run)
/// and what draws them (materials, images, models): nothing else of it
/// (its weapons, sounds, scripts, world) joins the match.
pub(crate) fn donor_fx_capture(dir: &Path, zone: &str, wanted: &std::collections::BTreeSet<String>) -> Result<asset_t6::ZoneCapture, String> {
    let mut c = asset_t6::capture_zone(&dir.join(format!("{zone}.ff"))).map_err(|e| e.to_string())?;
    let mut keep: std::collections::BTreeSet<String> = wanted.iter().map(|n| n.to_ascii_lowercase()).collect();
    loop {
        let before = keep.len();
        let mut more = Vec::new();
        for e in c.fx.iter().filter(|e| keep.contains(&e.name.to_ascii_lowercase())) {
            for el in &e.elems {
                for n in [&el.effect_on_impact, &el.effect_on_death, &el.effect_emitted, &el.effect_attached] {
                    if !n.is_empty() {
                        more.push(n.to_ascii_lowercase());
                    }
                }
                for v in &el.visuals {
                    if let asset_t6::FxVisualRef::Effect(n) = v {
                        more.push(n.to_ascii_lowercase());
                    }
                }
            }
        }
        keep.extend(more);
        if keep.len() == before {
            break;
        }
    }
    c.fx.retain(|e| keep.contains(&e.name.to_ascii_lowercase()));
    c.world = None;
    c.clip = None;
    c.primary_lights.clear();
    c.raw_files.clear();
    c.string_tables.clear();
    c.zbarriers.clear();
    c.localize.clear();
    c.path_nodes.clear();
    c.path_tables = Default::default();
    c.xanims.clear();
    c.map_ents.clear();
    c.weapons.clear();
    c.attachments.clear();
    c.attachment_uniques.clear();
    c.vehicles.clear();
    c.destructibles.clear();
    c.phys_presets.clear();
    c.phys_constraints.clear();
    c.weapon_camos.clear();
    c.fonts.clear();
    c.tracers.clear();
    c.impact_tables.clear();
    c.footstep_tables.clear();
    c.footstep_fx_tables.clear();
    c.sound_banks.clear();
    c.snd_globals = None;
    Ok(c)
}

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

#[cfg(test)]
mod sky_tint_tests {
    use super::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    #[test]
    fn grey_sky_turns_orange_and_keeps_its_scale() {
        // A 4x4 cube, every face grey with HDR scale 0.5 in alpha.
        let px = [128u8, 128, 128, 128];
        let data: Vec<u8> = std::iter::repeat_n(px, 16 * 6).flatten().collect();
        let sky = Image::new(
            Extent3d { width: 4, height: 4, depth_or_array_layers: 6 },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            bevy::asset::RenderAssetUsages::RENDER_WORLD,
        );
        let t = tinted_sky(&sky, [1.0, 0.52, 0.22], 0.7).unwrap();
        let d = t.data.as_ref().unwrap();
        assert_eq!(d.len(), 16 * 6 * 4);
        let (r, g, b, a) = (d[0], d[1], d[2], d[3]);
        assert!(r > g && g > b, "{r} {g} {b}");
        assert_eq!(a, 128);
    }
}

#[cfg(test)]
mod donor_fx_tests {
    /// Nuketown Zombies' fires and smoke, alone, when its zone is here
    /// (`IW4L_TEST_T6_ZONES`: the folder holding zm_nuked.ff).
    #[test]
    fn donor_keeps_only_the_wanted_effects() {
        let Ok(dir) = std::env::var("IW4L_TEST_T6_ZONES") else { return };
        let wanted = super::NUKED_FX_SWAPS.iter().map(|(_, to)| (*to).to_owned()).collect();
        let c = super::donor_fx_capture(std::path::Path::new(&dir), "zm_nuked", &wanted).unwrap();
        let names: Vec<&str> = c.fx.iter().map(|e| e.name.as_str()).collect();
        for (_, to) in super::NUKED_FX_SWAPS {
            assert!(names.iter().any(|n| n.eq_ignore_ascii_case(to)), "{to} missing from {names:?}");
        }
        assert!(c.weapons.is_empty() && c.world.is_none() && c.raw_files.is_empty());
        println!("{} effects kept: {names:?}", c.fx.len());
    }
}
