//! bo2mp: maps from other Call of Duty games in Black Ops II's map list.
//!
//! The installed games under `IW4L_GAMES` are scanned once for their
//! multiplayer maps (Modern Warfare 2, Modern Warfare 3, Black Ops; World at
//! War and Call of Duty 4 once their zones load). Each game gets its own
//! categories in the Change Map popup, like BO2's DLC packs: its base maps,
//! then its DLC maps, a page of at most `PAGE` maps each so no list runs off
//! the screen. Their rows go into BO2's `mp/mapstable.csv` (the faction
//! pairing borrowed from Aftermath) with their names in the text tables,
//! and the match setup learns which game's zone each one loads.

use std::sync::OnceLock;

/// The most maps a category lists before the next page starts.
const PAGE: usize = 14;

/// The first mapPackTypeIndex the extra categories use (BO2's own run to
/// its DLC5 and development indices).
const FIRST_PACK: u32 = 20;

/// The BO2 map whose mapstable row a foreign map's row copies.
const ROW_FROM: &str = "mp_la";

/// One game's maps: its zone prefix, its category key and title, its base
/// maps (in the order the game lists them) and its DLC maps, by zone name.
struct Game {
    prefix: &'static str,
    key: &'static str,
    title: &'static str,
    base: &'static [(&'static str, &'static str)],
    dlc: &'static [(&'static str, &'static str)],
}

const GAMES: &[Game] = &[
    Game {
        prefix: "iw4",
        key: "MW2",
        title: "Modern Warfare 2",
        base: &[
            ("mp_afghan", "Afghan"),
            ("mp_derail", "Derail"),
            ("mp_estate", "Estate"),
            ("mp_favela", "Favela"),
            ("mp_highrise", "Highrise"),
            ("mp_invasion", "Invasion"),
            ("mp_checkpoint", "Karachi"),
            ("mp_quarry", "Quarry"),
            ("mp_rundown", "Rundown"),
            ("mp_rust", "Rust"),
            ("mp_boneyard", "Scrapyard"),
            ("mp_nightshift", "Skidrow"),
            ("mp_subbase", "Sub Base"),
            ("mp_terminal", "Terminal"),
            ("mp_underpass", "Underpass"),
            ("mp_brecourt", "Wasteland"),
        ],
        dlc: &[
            ("mp_complex", "Bailout"),
            ("mp_crash", "Crash"),
            ("mp_overgrown", "Overgrown"),
            ("mp_compact", "Salvage"),
            ("mp_storm", "Storm"),
            ("mp_abandon", "Carnival"),
            ("mp_fuel2", "Fuel"),
            ("mp_strike", "Strike"),
            ("mp_trailerpark", "Trailer Park"),
            ("mp_vacant", "Vacant"),
        ],
    },
    Game {
        prefix: "iw5",
        key: "MW3",
        title: "Modern Warfare 3",
        base: &[
            ("mp_mogadishu", "Bakaara"),
            ("mp_alpha", "Lockdown"),
            ("mp_bootleg", "Bootleg"),
            ("mp_bravo", "Mission"),
            ("mp_carbon", "Carbon"),
            ("mp_dome", "Dome"),
            ("mp_exchange", "Downturn"),
            ("mp_hardhat", "Hardhat"),
            ("mp_interchange", "Interchange"),
            ("mp_lambeth", "Fallen"),
            ("mp_paris", "Resistance"),
            ("mp_plaza2", "Arkaden"),
            ("mp_radar", "Outpost"),
            ("mp_seatown", "Seatown"),
            ("mp_underground", "Underground"),
            ("mp_village", "Village"),
        ],
        dlc: &[
            ("mp_park", "Liberation"),
            ("mp_overwatch", "Overwatch"),
            ("mp_morningwood", "Black Box"),
            ("mp_italy", "Piazza"),
            ("mp_meteora", "Sanctuary"),
            ("mp_cement", "Foundation"),
            ("mp_qadeem", "Oasis"),
            ("mp_terminal_cls", "Terminal"),
            ("mp_hillside_ss", "Getaway"),
            ("mp_restrepo_ss", "Lookout"),
            ("mp_aground_ss", "Aground"),
            ("mp_courtyard_ss", "Erosion"),
            ("mp_crosswalk_ss", "Intersection"),
            ("mp_burn_ss", "U-Turn"),
            ("mp_six_ss", "Vortex"),
            ("mp_shipbreaker", "Decommission"),
            ("mp_roughneck", "Off Shore"),
            ("mp_moab", "Gulch"),
            ("mp_boardwalk", "Boardwalk"),
            ("mp_nola", "Parish"),
        ],
    },
    Game {
        prefix: "t5",
        key: "BO1",
        title: "Black Ops",
        base: &[
            ("mp_array", "Array"),
            ("mp_cracked", "Cracked"),
            ("mp_crisis", "Crisis"),
            ("mp_firingrange", "Firing Range"),
            ("mp_duga", "Grid"),
            ("mp_hanoi", "Hanoi"),
            ("mp_cairo", "Havana"),
            ("mp_havoc", "Jungle"),
            ("mp_cosmodrome", "Launch"),
            ("mp_nuked", "Nuketown"),
            ("mp_radiation", "Radiation"),
            ("mp_mountain", "Summit"),
            ("mp_villa", "Villa"),
            ("mp_russianbase", "WMD"),
        ],
        dlc: &[
            ("mp_berlinwall2", "Berlin Wall"),
            ("mp_discovery", "Discovery"),
            ("mp_kowloon", "Kowloon"),
            ("mp_stadium", "Stadium"),
            ("mp_gridlock", "Convoy"),
            ("mp_hotel", "Hotel"),
            ("mp_outskirts", "Stockpile"),
            ("mp_zoo", "Zoo"),
            ("mp_area51", "Hangar 18"),
            ("mp_silo", "Silo"),
            ("mp_drivein", "Drive-In"),
            ("mp_golfcourse", "Hazard"),
        ],
    },
    Game {
        prefix: "t4",
        key: "WAW",
        title: "World at War",
        base: &[
            ("mp_airfield", "Airfield"),
            ("mp_asylum", "Asylum"),
            ("mp_castle", "Castle"),
            ("mp_shrine", "Cliffside"),
            ("mp_courtyard", "Courtyard"),
            ("mp_dome", "Dome"),
            ("mp_downfall", "Downfall"),
            ("mp_hangar", "Hangar"),
            ("mp_makin", "Makin"),
            ("mp_outskirts", "Outskirts"),
            ("mp_roundhouse", "Roundhouse"),
            ("mp_seelow", "Seelow"),
            ("mp_suburban", "Upheaval"),
        ],
        dlc: &[
            ("mp_makin_day", "Makin Day"),
            ("mp_kneedeep", "Knee Deep"),
            ("mp_nachtfeuer", "Nightfire"),
            ("mp_subway", "Station"),
            ("mp_stalingrad", "Corrosion"),
            ("mp_docks", "Sub Pens"),
            ("mp_kwai", "Banzai"),
            ("mp_vodka", "Revolution"),
            ("mp_bgate", "Breach"),
            ("mp_drum", "Battery"),
        ],
    },
    Game {
        prefix: "iw3",
        key: "COD4",
        title: "Call of Duty 4",
        base: &[
            ("mp_convoy", "Ambush"),
            ("mp_backlot", "Backlot"),
            ("mp_bloc", "Bloc"),
            ("mp_bog", "Bog"),
            ("mp_countdown", "Countdown"),
            ("mp_crash", "Crash"),
            ("mp_crossfire", "Crossfire"),
            ("mp_citystreets", "District"),
            ("mp_farm", "Downpour"),
            ("mp_overgrown", "Overgrown"),
            ("mp_pipeline", "Pipeline"),
            ("mp_shipment", "Shipment"),
            ("mp_showdown", "Showdown"),
            ("mp_strike", "Strike"),
            ("mp_vacant", "Vacant"),
            ("mp_cargoship", "Wet Work"),
        ],
        dlc: &[
            ("mp_crash_snow", "Winter Crash"),
            ("mp_broadcast", "Broadcast"),
            ("mp_carentan", "Chinatown"),
            ("mp_creek", "Creek"),
            ("mp_killhouse", "Killhouse"),
        ],
    },
];

/// A map from another game, as the menus list it.
#[derive(Clone, Debug)]
pub(crate) struct ForeignMap {
    /// The name the menus know it by (`ui_mapname`): its zone name, with
    /// the game's prefix added when another game has a map by that name.
    pub ui_name: String,
    pub game: &'static str,
    pub stem: String,
    pub title: String,
    /// The game it comes from (`Modern Warfare 2 (DLC)`).
    pub origin: String,
}

/// A category in the Change Map popup.
#[derive(Clone, Debug)]
pub(crate) struct Category {
    /// `MPUI_MAP_CATEGORY_<name>_CAPS`.
    pub name: String,
    pub title: String,
    pub description: String,
    pub pack: u32,
    pub maps: Vec<ForeignMap>,
}

/// The categories, found once (an empty list when no other game is
/// installed).
pub(crate) fn categories() -> &'static [Category] {
    static FOUND: OnceLock<Vec<Category>> = OnceLock::new();
    FOUND.get_or_init(|| {
        let categories = discover();
        session::bo2mp_setup::register_foreign_maps(
            categories
                .iter()
                .flat_map(|c| c.maps.iter())
                .map(|m| (m.ui_name.clone(), m.game.to_owned(), m.stem.clone()))
                .collect(),
        );
        hks_t6::host::set_extra_map_categories(
            categories
                .iter()
                .map(|c| (c.name.clone(), c.pack, "menu_mp_map_category_dlc0".to_owned()))
                .collect(),
        );
        diag::info!(
            Ui,
            "bo2mp maps: {} more categories: {}",
            categories.len(),
            categories
                .iter()
                .map(|c| format!("{} ({})", c.title, c.maps.len()))
                .collect::<Vec<_>>()
                .join(", ")
        );
        categories
    })
}

fn title_of(stem: &str) -> String {
    stem.trim_start_matches("mp_")
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// bo2mp: a usermap's title from its zone name, a source game tag moved to
/// the end (`mp_waw_castle` -> `Castle (WaW)`).
fn usermap_title(stem: &str) -> String {
    const TAGS: &[(&str, &str)] = &[
        ("waw", "WaW"),
        ("t4", "WaW"),
        ("cod4", "CoD4"),
        ("iw3", "CoD4"),
        ("cod2", "CoD2"),
        ("cod1", "CoD"),
        ("uo", "UO"),
        ("bo1", "BO1"),
        ("bo2", "BO2"),
        ("bo3", "BO3"),
        ("t5", "BO1"),
        ("mw1", "MW1"),
        ("mw2", "MW2"),
        ("mw3", "MW3"),
        ("mwr", "MWR"),
        ("codo", "CoDO"),
        ("ghosts", "Ghosts"),
    ];
    let words: Vec<&str> = stem.trim_start_matches("mp_").split('_').filter(|w| !w.is_empty()).collect();
    let tag_of = |w: &str| TAGS.iter().find(|(t, _)| w.eq_ignore_ascii_case(t)).map(|(_, n)| *n);
    let (tag, rest) = match (words.first().copied().and_then(tag_of), words.last().copied().and_then(tag_of)) {
        (Some(tag), _) if words.len() > 1 => (Some(tag), &words[1..]),
        (_, Some(tag)) if words.len() > 1 => (Some(tag), &words[..words.len() - 1]),
        _ => (None, &words[..]),
    };
    let title = title_of(&rest.join("_"));
    match tag {
        Some(tag) => format!("{title} ({tag})"),
        None => title,
    }
}

fn discover() -> Vec<Category> {
    let Ok(root) = asset_transport::games_root_from_env() else {
        return Vec::new();
    };
    let packs = asset_transport::list_mp_map_packs(&root);
    // bo2mp: usermaps (`<game>/usermaps/<map>/<map>.ff`), by `game:map`.
    let custom: Vec<String> = packs
        .iter()
        .filter(|p| p.label.ends_with(asset_transport::USERMAPS_PACK))
        .flat_map(|p| p.maps.iter().cloned())
        .collect();
    if !custom.is_empty() {
        diag::info!(Ui, "bo2mp maps: usermaps {}", custom.join(", "));
    }
    // Every map zone of every installed game but Black Ops II's own.
    let mut installed: Vec<(String, String)> = packs
        .into_iter()
        .flat_map(|p| p.maps)
        .filter_map(|key| {
            let (game, stem) = key.split_once(':')?;
            (game != "t6").then(|| (game.to_owned(), stem.to_owned()))
        })
        .collect();
    // Call of Duty 4's and World at War's maps (read by their own readers:
    // `iw3`, and `t4` through the CoD4 lane).
    installed.extend(
        asset_transport::list_legacy_mp_maps(&root)
            .into_iter()
            .filter(|(game, _)| game == "iw3" || game == "t4"),
    );
    installed.sort();
    installed.dedup();
    // bo2mp: a Call of Duty 4 map MW2's folder also has (IW4x's CoD4 pack:
    // Backlot, Crossfire, Wet Work, ...) is played as MW2's port - it loads
    // through MW2's lane like the rest of MW2 - so CoD4's own copy is not
    // listed.
    let ported: Vec<String> = installed
        .iter()
        .filter(|(g, s)| g == "iw3" && installed.iter().any(|(g2, s2)| g2 == "iw4" && s2 == s))
        .map(|(_, s)| s.clone())
        .collect();
    if !ported.is_empty() {
        diag::info!(Ui, "bo2mp maps: {} Call of Duty 4 maps played as MW2's ports instead: {}", ported.len(), ported.join(", "));
        installed.retain(|(g, s)| !(g == "iw3" && ported.contains(s)));
    }
    let shared = |stem: &str| installed.iter().filter(|(_, s)| s == stem).count() > 1;
    // Every game's maps in one list (base maps, then DLC, game by game),
    // split into sections of at most `PAGE` so no list runs off the screen.
    let mut all: Vec<ForeignMap> = Vec::new();
    for game in GAMES {
        let is_custom = |stem: &str| custom.iter().any(|c| c.split_once(':') == Some((game.prefix, stem)));
        let all_mine: Vec<&String> = installed.iter().filter(|(g, _)| g == game.prefix).map(|(_, s)| s).collect();
        if all_mine.is_empty() {
            continue;
        }
        let (custom_mine, mine): (Vec<&String>, Vec<&String>) = all_mine.into_iter().partition(|s| is_custom(s));
        let known = |list: &[(&str, &str)]| -> Vec<(String, String)> {
            list.iter()
                .filter(|(s, _)| mine.iter().any(|m| m.as_str() == *s))
                .map(|(s, t)| ((*s).to_owned(), (*t).to_owned()))
                .collect()
        };
        let base = known(game.base);
        let mut dlc = known(game.dlc);
        // Another game's map ported into this one (IW4x's CoD4 and MW3
        // packs in MW2): its own title, listed as that game's map.
        let mut ports: Vec<(String, String, &'static str)> = Vec::new();
        // Maps the tables do not name (a remaster's extra, a mod map) go
        // with the DLC, by their zone names.
        for stem in &mine {
            if game.base.iter().chain(game.dlc).any(|(s, _)| *s == stem.as_str()) {
                continue;
            }
            let from = GAMES
                .iter()
                .filter(|g| g.prefix != game.prefix)
                .find_map(|g| g.base.iter().chain(g.dlc).find(|(s, _)| *s == stem.as_str()).map(|(_, t)| ((*t).to_owned(), g.title)));
            match from {
                Some((title, origin)) => ports.push(((*stem).clone(), title, origin)),
                None => dlc.push(((*stem).clone(), title_of(stem))),
            }
        }
        // Usermaps last, titled from their zone names.
        let custom_maps: Vec<(String, String)> =
            custom_mine.iter().map(|s| ((*s).clone(), usermap_title(s))).collect();
        for (stem, title, origin) in ports {
            all.push(ForeignMap {
                ui_name: if shared(&stem) { format!("{stem}_{}", game.prefix) } else { stem.clone() },
                game: game.prefix,
                stem,
                title,
                origin: format!("{origin} (in {})", game.title),
            });
        }
        for (maps, suffix) in [(base, ""), (dlc, " (DLC)"), (custom_maps, " (Custom)")] {
            for (stem, title) in maps {
                all.push(ForeignMap {
                    ui_name: if shared(&stem) { format!("{stem}_{}", game.prefix) } else { stem.clone() },
                    game: game.prefix,
                    stem,
                    title,
                    origin: format!("{}{suffix}", game.title),
                });
            }
        }
    }
    let pages: Vec<&[ForeignMap]> = all.chunks(PAGE).collect();
    let numerals = ["", " II", " III", " IV", " V", " VI", " VII", " VIII", " IX", " X"];
    pages
        .iter()
        .enumerate()
        .map(|(i, chunk)| {
            let mut games: Vec<&str> = Vec::new();
            for m in chunk.iter() {
                let g = GAMES.iter().find(|g| g.prefix == m.game).map_or("", |g| g.title);
                if !games.contains(&g) {
                    games.push(g);
                }
            }
            Category {
                name: format!("BO2RB_CLASSIC{}", i + 1),
                title: format!("CLASSIC MAPS{}", numerals.get(i).copied().unwrap_or("")),
                description: format!("{}: {} maps.", games.join(", "), chunk.len()),
                pack: FIRST_PACK + i as u32,
                maps: chunk.to_vec(),
            }
        })
        .collect()
}

#[cfg(test)]
mod usermap_title_tests {
    #[test]
    fn titles() {
        assert_eq!(super::usermap_title("mp_waw_castle"), "Castle (WaW)");
        assert_eq!(super::usermap_title("mp_castle_waw"), "Castle (WaW)");
        assert_eq!(super::usermap_title("mp_cod4_crash_snow"), "Crash Snow (CoD4)");
        assert_eq!(super::usermap_title("mp_nuketown"), "Nuketown");
        assert_eq!(super::usermap_title("mp_waw"), "Waw");
    }
}

/// The text key of a foreign map's name and description.
fn name_key(map: &ForeignMap) -> String {
    format!("BO2RB_MAP_{}", map.ui_name.to_ascii_uppercase())
}

/// Puts the foreign maps into BO2's tables and text: a mapstable row each
/// (Aftermath's, renamed), and each map's and category's text.
pub(crate) fn inject(values: &mut hks_t6::host::EngineValues) {
    let categories = categories();
    if categories.is_empty() {
        return;
    }
    for c in categories {
        values
            .localize
            .insert(format!("MPUI_MAP_CATEGORY_{}_CAPS", c.name), c.title.clone());
        values
            .localize
            .insert(format!("MPUI_MAP_CATEGORY_{}_DESC", c.name), c.description.clone());
        for m in &c.maps {
            values.localize.insert(name_key(m), m.title.to_uppercase());
            values.localize.insert(format!("{}_DESC", name_key(m)), format!("{} from {}.", m.title, m.origin));
        }
    }
    let Some(rows) = values.tables.get_mut("mp/mapstable.csv") else {
        return;
    };
    let Some(template) = rows.iter().find(|r| r.first().is_some_and(|c| c == ROW_FROM)).cloned() else {
        return;
    };
    let mut sort = 100;
    for c in categories {
        for m in &c.maps {
            if rows.iter().any(|r| r.first() == Some(&m.ui_name)) {
                continue;
            }
            let mut row = template.clone();
            row.resize(row.len().max(12), String::new());
            row[0] = m.ui_name.clone();
            row[3] = name_key(m);
            row[5] = sort.to_string();
            row[6] = format!("{}_DESC", name_key(m));
            row[11] = c.pack.to_string();
            sort += 1;
            rows.push(row);
        }
    }
}

/// A picture read off the main thread once and kept by `key`: `None` while
/// it is being read (and for good when nothing was found).
pub(crate) fn picture_async(
    key: &str,
    srgb: bool,
    read: impl FnOnce() -> Option<(u32, u32, Vec<u8>)> + Send + 'static,
) -> Option<std::sync::Arc<bevy::prelude::Image>> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    type Read = HashMap<String, Option<Arc<bevy::prelude::Image>>>;
    static READ: Mutex<Option<Read>> = Mutex::new(None);
    let mut guard = READ.lock().ok()?;
    let done = guard.get_or_insert_with(HashMap::new);
    if let Some(found) = done.get(key) {
        return found.clone();
    }
    done.insert(key.to_owned(), None);
    let key = key.to_owned();
    std::thread::Builder::new()
        .name("ui-picture".into())
        .spawn(move || {
            let image = read().map(|(w, h, rgba)| {
                use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
                Arc::new(bevy::prelude::Image::new(
                    Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    TextureDimension::D2,
                    rgba,
                    if srgb { TextureFormat::Rgba8UnormSrgb } else { TextureFormat::Rgba8Unorm },
                    bevy::asset::RenderAssetUsages::default(),
                ))
            });
            diag::info!(Ui, "picture `{key}`: {}", if image.is_some() { "read from its game" } else { "not found" });
            if let Ok(mut guard) = READ.lock()
                && let Some(done) = guard.as_mut()
            {
                done.insert(key, image);
            }
        })
        .ok();
    None
}

/// The namespace of a game prefix (`iw4` -> MW2).
fn namespace_of(prefix: &str) -> Option<asset_core::AssetNamespace> {
    Some(match prefix {
        "iw4" => asset_core::AssetNamespace::Iw4,
        "iw5" => asset_core::AssetNamespace::Iw5,
        "t5" => asset_core::AssetNamespace::T5,
        _ => return None,
    })
}

/// An image from a game's own IWDs (the first of `names` it has).
pub(crate) fn read_game_image(prefix: &str, names: &[String]) -> Option<(u32, u32, Vec<u8>)> {
    let root = asset_transport::games_root_from_env().ok()?;
    let main = match asset_transport::legacy_version(prefix) {
        // Call of Duty 4 / World at War: the `main` beside their zones.
        Some(version) => {
            let common = asset_transport::find_zone_file_version(&root, "common_mp", version).ok()?;
            asset_transport::game_root_for_zone(&common.path).ok()?.join("main")
        }
        None => {
            let trees = asset_transport::NamespaceTrees::discover(&root);
            trees.main_for(namespace_of(prefix)?)?.to_path_buf()
        }
    };
    names
        .iter()
        .find_map(|n| asset_material::decode_ui_image_from_main(&main, n).ok().flatten())
}

/// A prestige icon from another game where BO2's menus ask for one: the
/// combined prestiges' `<game>:<material>` (bo2_profile's progression),
/// read from that game's own files, the first of its candidate names it
/// has (logged, as the names are not all known for certain).
pub(crate) fn prestige_picture(name: &str) -> Option<std::sync::Arc<bevy::prelude::Image>> {
    let (game, names) = bo2_profile::progression::foreign_picture(name)?;
    let key = name.to_owned();
    picture_async(name, true, move || {
        let found = names.iter().find_map(|n| read_game_image(game, std::slice::from_ref(n)).map(|img| (n, img)));
        match &found {
            Some((n, _)) => diag::info!(Ui, "bo2mp progression: `{key}` is {game}'s `{n}`"),
            None => diag::warn!(Ui, "bo2mp progression: `{key}`: {game} has none of {}", names.join(", ")),
        }
        found.map(|(_, img)| img)
    })
}

/// A foreign map's picture where BO2's menus ask for one: its map-select
/// picture (`menu_<map>_map_select_final`), its loading picture
/// (`loadscreen_<map>`) or its preview, read from the map's own game (its
/// loading screen, else its preview).
pub(crate) fn map_picture(material: &str) -> Option<std::sync::Arc<bevy::prelude::Image>> {
    let ui_name = material
        .strip_prefix("menu_")
        .and_then(|m| m.strip_suffix("_map_select_final"))
        .or_else(|| material.strip_prefix("loadscreen_"))
        .or_else(|| material.strip_prefix("menu_mp_map_select_").and_then(|m| m.strip_suffix("_big")))?;
    let map = categories().iter().flat_map(|c| c.maps.iter()).find(|m| m.ui_name == ui_name)?.clone();
    picture_async(material, true, move || {
        read_game_image(
            map.game,
            &[
                format!("loadscreen_{}", map.stem),
                format!("preview_{}", map.stem),
                format!("{}_load", map.stem),
            ],
        )
    })
}
