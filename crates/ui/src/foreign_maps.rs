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

/// bo2mp: Infinite Warfare's free re-releases of its DLC maps and the
/// originals they repeat: a variant is listed only when its original is not
/// installed (the zones are the same maps, a few vertices apart).
const IW7_VARIANTS: &[(&str, &str)] = &[
    ("mp_renaissance2", "mp_geneva"),
    ("mp_carnage2", "mp_rally"),
    ("mp_permafrost2", "mp_overflow"),
    ("mp_turista2", "mp_marsoasis"),
];

/// bo2mp: IW4x's own map pack in Modern Warfare 2 (`zone/iw4x/.../dlc`):
/// Call of Duty 4's and Black Ops' maps rebuilt for MW2, and its variants,
/// titled as IW4x lists them. They are MW2's maps here (MW2's section), and
/// a Call of Duty 4 map with a version in this list is not listed from
/// Call of Duty 4 itself (`COD4_IN_MW2`).
const IW4X: &[(&str, &str)] = &[
    ("mp_nuked", "Nuketown"),
    ("mp_cross_fire", "Crossfire"),
    ("mp_cargoship", "Wet Work"),
    ("mp_bloc_sh", "Forgotten City"),
    ("mp_killhouse", "Killhouse"),
    ("mp_bog_sh", "Bog"),
    ("mp_cargoship_sh", "Freighter"),
    ("mp_shipment", "Shipment"),
    ("mp_shipment_long", "Shipment: Long"),
    ("mp_rust_long", "Rust: Long"),
    ("mp_firingrange", "Firing Range"),
    ("mp_bloc", "Bloc"),
    ("mp_crash_tropical", "Crash: Tropical"),
    ("mp_estate_tropical", "Estate: Tropical"),
    ("mp_fav_tropical", "Favela: Tropical"),
    ("mp_storm_spring", "Chemical Plant"),
    ("mp_convoy", "Ambush"),
    ("mp_backlot", "Backlot"),
    ("mp_countdown", "Countdown"),
    ("mp_citystreets", "District"),
    ("mp_farm", "Downpour"),
    ("mp_pipeline", "Pipeline"),
    ("mp_showdown", "Showdown"),
    ("mp_crash_snow", "Winter Crash"),
    ("mp_broadcast", "Broadcast"),
    ("mp_carentan", "Chinatown"),
];

/// bo2mp: MW2's maps that are Call of Duty 4's (IW4x's rebuilds, its
/// variants of them, and MW2's own Stimulus remakes): listed in Call of
/// Duty 4's section, played as MW2's.
const COD4_MAPS_IN_MW2: &[&str] = &[
    "mp_convoy", "mp_backlot", "mp_bloc", "mp_bloc_sh", "mp_bog_sh", "mp_cargoship", "mp_cargoship_sh",
    "mp_carentan", "mp_citystreets", "mp_countdown", "mp_crash_snow", "mp_cross_fire", "mp_farm",
    "mp_killhouse", "mp_pipeline", "mp_shipment", "mp_shipment_long", "mp_showdown", "mp_broadcast",
    "mp_crash", "mp_overgrown", "mp_strike", "mp_vacant", "mp_creek",
];

/// bo2mp: Call of Duty 4 maps whose MW2 version has another zone name.
const COD4_IN_MW2: &[(&str, &str)] = &[("mp_bog", "mp_bog_sh"), ("mp_crossfire", "mp_cross_fire")];

/// The MW2 zone that is Call of Duty 4's `stem` (the same name, or
/// `COD4_IN_MW2`'s).
fn mw2_version_of(stem: &str) -> &str {
    COD4_IN_MW2.iter().find(|(c, _)| *c == stem).map_or(stem, |(_, m)| m)
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
    // bo2mp: Call of Duty: Ghosts (its maps through `assets::lane::iw6_map`).
    Game {
        prefix: "iw6",
        key: "GHOSTS",
        title: "Ghosts",
        base: &[
            ("mp_chasm", "Chasm"),
            ("mp_flooded", "Flooded"),
            ("mp_frag", "Freight"),
            ("mp_descent_new", "Free Fall"),
            ("mp_dart", "Octane"),
            ("mp_zebra", "Overlord"),
            ("mp_prisonbreak", "Prison Break"),
            ("mp_hashima", "Siege"),
            ("mp_sovereign", "Sovereign"),
            ("mp_skeleton", "Stonehaven"),
            ("mp_fahrenheit", "Stormfront"),
            ("mp_strikezone", "Strikezone"),
            ("mp_lonestar", "Tremor"),
            ("mp_warhawk", "Warhawk"),
            ("mp_snow", "Whiteout"),
        ],
        dlc: &[
            ("mp_swamp", "Fog"),
            ("mp_ca_rumble", "Bayview"),
            ("mp_ca_red_river", "Containment"),
            ("mp_boneyard_ns", "Ignition"),
            ("mp_battery3", "Ruins"),
            ("mp_ca_impact", "Collision"),
            ("mp_ca_behemoth", "Behemoth"),
            ("mp_dome_ns", "Unearthed"),
            ("mp_dig", "Pharaoh"),
            ("mp_favela_iw6", "Favela"),
            ("mp_pirate", "Mutiny"),
            ("mp_zulu", "Departed"),
            ("mp_zerosub", "Subzero"),
            ("mp_conflict", "Dynasty"),
            ("mp_mine", "Goldrush"),
            ("mp_shipment_ns", "Showtime"),
        ],
    },
    // bo2mp: Call of Duty: Infinite Warfare (its maps through
    // `assets::lane::iw7`). Titles as the game's own English text names
    // them (`presence_<map>` in `english/eng_code_post_gfx.ff`, past the
    // zone's font); `mp_dome_iw` is the `mp_dome` there. The free re-releases
    // of DLC maps (`IW7_VARIANTS`) are not listed when their originals are
    // installed.
    Game {
        prefix: "iw7",
        key: "IW",
        title: "Infinite Warfare",
        base: &[
            ("mp_parkour", "Breakout"),
            ("mp_quarry", "Crusher"),
            ("mp_frontier", "Frontier"),
            ("mp_proto", "Frost"),
            ("mp_dome_iw", "Genesis"),
            ("mp_breakneck", "Mayday"),
            ("mp_metropolis", "Precinct"),
            ("mp_riot", "Retaliation"),
            ("mp_divide", "Scorch"),
            ("mp_rivet", "SkyDock"),
            ("mp_fallen", "Throwback"),
            ("mp_skyway", "Terminal"),
        ],
        dlc: &[
            ("mp_flip", "Excess"),
            ("mp_desert", "Grounded"),
            ("mp_afghan", "Dominion"),
            ("mp_prime", "Noir"),
            ("mp_geneva", "Renaissance"),
            ("mp_neon", "Neon"),
            ("mp_mansion", "Archive"),
            ("mp_paris", "Ember"),
            ("mp_junk", "Scrap"),
            ("mp_marsoasis", "Turista"),
            ("mp_nova", "Bermuda"),
            ("mp_pixel", "Fore"),
            ("mp_overflow", "Permafrost"),
            ("mp_codphish", "Altitude"),
            ("mp_rally", "Carnage"),
            ("mp_depot", "Depot 22"),
            ("mp_hawkwar", "Heartland"),
            ("mp_renaissance2", "Renaissance"),
            ("mp_carnage2", "Carnage"),
            ("mp_permafrost2", "Permafrost"),
            ("mp_turista2", "Turista"),
        ],
    },
    // bo2mp: Call of Duty: Black Ops III (its maps through
    // `assets::lane::t7_map`).
    Game {
        prefix: "t7",
        key: "BO3",
        title: "Black Ops III",
        base: &[
            ("mp_biodome", "Aquarium"),
            ("mp_spire", "Breach"),
            ("mp_sector", "Combine"),
            ("mp_apartments", "Evac"),
            ("mp_chinatown", "Exodus"),
            ("mp_veiled", "Fringe"),
            ("mp_havoc", "Havoc"),
            ("mp_ethiopia", "Hunted"),
            ("mp_infection", "Infection"),
            ("mp_metro", "Metro"),
            ("mp_nuketown_x", "Nuk3town"),
            ("mp_redwood", "Redwood"),
            ("mp_stronghold", "Stronghold"),
        ],
        dlc: &[
            ("mp_redwood_ice", "Redwood Snow"),
            ("mp_veiled_heyday", "Fringe Night"),
            ("mp_crucible", "Gauntlet"),
            ("mp_rise", "Rise"),
            ("mp_skyjacked", "Skyjacked"),
            ("mp_waterpark", "Splash"),
            ("mp_conduit", "Rift"),
            ("mp_kung_fu", "Knockout"),
            ("mp_aerospace", "Spire"),
            ("mp_banzai", "Verge"),
            ("mp_arena", "Rumble"),
            ("mp_cryogen", "Cryogen"),
            ("mp_rome", "Empire"),
            ("mp_ruins", "Berserk"),
            ("mp_shrine", "Citadel"),
            ("mp_miniature", "Micro"),
            ("mp_western", "Outlaw"),
            ("mp_city", "Rupture"),
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
    /// The map picker's section: the game it was made for (`MODERN
    /// WARFARE 3` for MW3's maps ported into MW2), or `CUSTOM MAPS`.
    pub section: String,
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
    // `iw3`, and `t4` through the CoD4 lane), and Ghosts' (`iw6`, read by
    // `fastfile_iw6`).
    installed.extend(
        asset_transport::list_legacy_mp_maps(&root)
            .into_iter()
            .filter(|(game, _)| game == "iw3" || game == "t4" || game == "iw6" || game == "iw7" || game == "t7"),
    );
    installed.sort();
    installed.dedup();
    // bo2mp: a Call of Duty 4 map MW2's folder also has (IW4x's CoD4 pack:
    // Backlot, Crossfire, Wet Work, ...) is played as MW2's version - it
    // loads through MW2's lane like the rest of MW2, and runs better - so
    // CoD4's own copy is not listed.
    let ported: Vec<String> = installed
        .iter()
        .filter(|(g, s)| g == "iw3" && installed.iter().any(|(g2, s2)| g2 == "iw4" && s2 == mw2_version_of(s)))
        .map(|(_, s)| s.clone())
        .collect();
    if !ported.is_empty() {
        diag::info!(Ui, "bo2mp maps: {} Call of Duty 4 maps played as MW2's ports instead: {}", ported.len(), ported.join(", "));
        installed.retain(|(g, s)| !(g == "iw3" && ported.contains(s)));
    }
    // bo2mp: likewise a Modern Warfare 3 map IW4x has rebuilt for MW2
    // (Lockdown, Dome, Seatown, ...): MW2's version is played (it looks
    // better and wears MW2's soldiers), listed under MW3 with its MW3 name.
    let mw3_ported: Vec<String> = installed
        .iter()
        .filter(|(g, s)| g == "iw5" && installed.iter().any(|(g2, s2)| g2 == "iw4" && s2 == s))
        .map(|(_, s)| s.clone())
        .collect();
    if !mw3_ported.is_empty() {
        diag::info!(Ui, "bo2mp maps: {} Modern Warfare 3 maps played as MW2's ports instead: {}", mw3_ported.len(), mw3_ported.join(", "));
        installed.retain(|(g, s)| !(g == "iw5" && mw3_ported.contains(s)));
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
        let (custom_mine, mut mine): (Vec<&String>, Vec<&String>) = all_mine.into_iter().partition(|s| is_custom(s));
        if game.prefix == "iw7" {
            let has = |stem: &str| mine.iter().any(|m| m.as_str() == stem);
            let hidden: Vec<&str> = IW7_VARIANTS.iter().filter(|(v, original)| has(v) && has(original)).map(|(v, _)| *v).collect();
            mine.retain(|m| !hidden.contains(&m.as_str()));
        }
        let known = |list: &[(&str, &str)]| -> Vec<(String, String)> {
            list.iter()
                .filter(|(s, _)| mine.iter().any(|m| m.as_str() == *s))
                .map(|(s, t)| ((*s).to_owned(), (*t).to_owned()))
                .collect()
        };
        let base = known(game.base);
        let mut dlc = known(game.dlc);
        // IW4x's pack: MW2's own maps, with IW4x's titles.
        let iw4x: &[(&str, &str)] = if game.prefix == "iw4" { IW4X } else { &[] };
        dlc.extend(known(iw4x));
        // Another game's map ported into this one (IW4x's CoD4 and MW3
        // packs in MW2): its own title, listed as that game's map.
        let mut ports: Vec<(String, String, &'static str)> = Vec::new();
        // Maps the tables do not name (a remaster's extra, a mod map) go
        // with the DLC, by their zone names.
        for stem in &mine {
            if game.base.iter().chain(game.dlc).chain(iw4x).any(|(s, _)| *s == stem.as_str()) {
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
                section: origin.to_uppercase(),
            });
        }
        for (maps, suffix) in [(base, ""), (dlc, " (DLC)"), (custom_maps, " (Custom)")] {
            for (stem, title) in maps {
                let cod4 = game.prefix == "iw4" && suffix != " (Custom)" && COD4_MAPS_IN_MW2.contains(&stem.as_str());
                all.push(ForeignMap {
                    ui_name: if shared(&stem) { format!("{stem}_{}", game.prefix) } else { stem.clone() },
                    game: game.prefix,
                    title,
                    origin: if cod4 { "Call of Duty 4 (MW2 version)".to_owned() } else { format!("{}{suffix}", game.title) },
                    section: if suffix == " (Custom)" {
                        "CUSTOM MAPS".to_owned()
                    } else if cod4 {
                        "CALL OF DUTY 4".to_owned()
                    } else {
                        game.title.to_uppercase()
                    },
                    stem,
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

    #[test]
    fn cod4_maps_have_their_mw2_versions() {
        assert_eq!(super::mw2_version_of("mp_bog"), "mp_bog_sh");
        assert_eq!(super::mw2_version_of("mp_backlot"), "mp_backlot");
        // No title twice in IW4x's list.
        let mut t: Vec<&str> = super::IW4X.iter().map(|(s, _)| *s).collect();
        t.sort();
        t.dedup();
        assert_eq!(t.len(), super::IW4X.len());
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

type PictureMap = std::collections::HashMap<String, Option<std::sync::Arc<bevy::prelude::Image>>>;
static PICTURES: std::sync::Mutex<Option<PictureMap>> = std::sync::Mutex::new(None);

/// Marks `key` as being read; false when it is read or being read already.
fn claim_picture(key: &str) -> bool {
    let Ok(mut guard) = PICTURES.lock() else { return false };
    let done = guard.get_or_insert_with(Default::default);
    if done.contains_key(key) {
        return false;
    }
    done.insert(key.to_owned(), None);
    true
}

fn store_picture(key: String, image: Option<std::sync::Arc<bevy::prelude::Image>>) {
    if let Ok(mut guard) = PICTURES.lock() {
        guard.get_or_insert_with(Default::default).insert(key, image);
    }
}

fn picture_image((w, h, rgba): (u32, u32, Vec<u8>), srgb: bool) -> std::sync::Arc<bevy::prelude::Image> {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    std::sync::Arc::new(bevy::prelude::Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        rgba,
        if srgb { TextureFormat::Rgba8UnormSrgb } else { TextureFormat::Rgba8Unorm },
        bevy::asset::RenderAssetUsages::default(),
    ))
}

/// A picture read off the main thread once and kept by `key`: `None` while
/// it is being read (and for good when nothing was found).
pub(crate) fn picture_async(
    key: &str,
    srgb: bool,
    read: impl FnOnce() -> Option<(u32, u32, Vec<u8>)> + Send + 'static,
) -> Option<std::sync::Arc<bevy::prelude::Image>> {
    {
        let guard = PICTURES.lock().ok()?;
        if let Some(found) = guard.as_ref().and_then(|d| d.get(key)) {
            return found.clone();
        }
    }
    if !claim_picture(key) {
        return None;
    }
    let key = key.to_owned();
    std::thread::Builder::new()
        .name("ui-picture".into())
        .spawn(move || {
            let image = read().map(|p| picture_image(p, srgb));
            diag::info!(Ui, "picture `{key}`: {}", if image.is_some() { "read from its game" } else { "not found" });
            store_picture(key, image);
        })
        .ok();
    None
}

/// bo2mp: which other games' guns Create-a-Class lists (an installed game
/// with its multiplayer common zone; off with `BO2MP_GUEST_GUNS=0`).
pub(crate) fn guest_guns_available() {
    let on = std::env::var("BO2MP_GUEST_GUNS").map_or(true, |v| v.trim() != "0");
    let root = asset_transport::games_root_from_env().ok();
    // (Black Ops' zone version, as its common_mp carries it.)
    let t5 = on && root.as_ref().is_some_and(|r| asset_transport::find_common_mp_for_envelope(r, 0x1D9).is_ok());
    // (Modern Warfare 2's zone version.)
    let iw4 = on && root.as_ref().is_some_and(|r| asset_transport::find_common_mp_for_envelope(r, 0x114).is_ok());
    bo2_profile::guest::set_available("t5", t5);
    bo2_profile::guest::set_available("iw4", iw4);
    let say = |listed: bool| if listed { "guns listed" } else if on { "not installed" } else { "off (BO2MP_GUEST_GUNS=0)" };
    diag::info!(Ui, "bo2mp guest guns: Black Ops {}, Modern Warfare 2 {}", say(t5), say(iw4));
    // (Modern Warfare 3's zone version is 1; World at War's and CoD4's are
    // `legacy_version`'s.)
    let legacy = |game| asset_transport::legacy_version(game).unwrap_or(u32::MAX);
    for (game, version, title) in
        [("iw5", 1, "Modern Warfare 3"), ("t4", legacy("t4"), "World at War"), ("iw3", legacy("iw3"), "Call of Duty 4")]
    {
        let listed = on && root.as_ref().is_some_and(|r| asset_transport::find_common_mp_for_envelope(r, version).is_ok());
        bo2_profile::guest::set_available(game, listed);
        diag::info!(Ui, "bo2mp guest guns: {title} {}", say(listed));
    }
    // (Ghosts: its zones are read by `fastfile_iw6`; its guns are in
    // common_core_mp.)
    let ghosts = on && ghosts_install(root.as_ref()).is_some_and(|i| i.join("zone").join("common_core_mp.ff").is_file());
    bo2_profile::guest::set_available("iw6", ghosts);
    diag::info!(Ui, "bo2mp guest guns: Ghosts {}", say(ghosts));
    // (Advanced Warfare: read by `fastfile_s1`; its guns are in common_mp.)
    let extra: Vec<std::path::PathBuf> = asset_transport::game_paths::get("aw").into_iter().collect();
    let aw = on && fastfile_s1::install::find_install(root.as_ref().map(|r| r.0.as_path()), &extra).is_some();
    bo2_profile::guest::set_available("s1", aw);
    diag::info!(Ui, "bo2mp guest guns: Advanced Warfare {}", say(aw));
    // (Black Ops III: read by `fastfile_t7`; its guns are in core_common.)
    let bo3 = on
        && root.as_ref().is_some_and(|r| {
            asset_transport::find_zone_file_version(r, "core_common", asset_transport::T7_ZONE_VERSION).is_ok()
        });
    bo2_profile::guest::set_available("t7", bo3);
    diag::info!(Ui, "bo2mp guest guns: Black Ops III {}", say(bo3));
}

/// bo2mp: Black Ops III's weapon icons (`t7_icon_weapon_*` in core_common),
/// read once a run (the zone is big: it is walked once, its icons kept).
fn bo3_icons() -> Option<&'static std::collections::HashMap<String, (u32, u32, Vec<u8>)>> {
    static ICONS: std::sync::OnceLock<Option<std::collections::HashMap<String, (u32, u32, Vec<u8>)>>> =
        std::sync::OnceLock::new();
    ICONS
        .get_or_init(|| {
            let root = asset_transport::games_root_from_env().ok()?;
            let zone = asset_transport::find_zone_file_version(&root, "core_common", asset_transport::T7_ZONE_VERSION).ok()?;
            match fastfile_t7::ui_image::read_images(&zone.path, |n| n.starts_with("t7_icon_weapon_"), 512) {
                Ok(list) => {
                    diag::info!(Ui, "bo2mp: {} Black Ops III weapon icons read", list.len());
                    Some(list.into_iter().map(|(n, w, h, px)| (n, (w, h, px))).collect())
                }
                Err(e) => {
                    diag::warn!(Ui, "bo2mp: Black Ops III weapon icons: {e}");
                    None
                }
            }
        })
        .as_ref()
}

/// bo2mp: where Ghosts is installed (`fastfile_iw6::install`'s search, with
/// game_paths.cfg's `ghosts=`).
fn ghosts_install(root: Option<&asset_transport::GamesRoot>) -> Option<std::path::PathBuf> {
    let extra: Vec<std::path::PathBuf> = asset_transport::game_paths::get("ghosts").into_iter().collect();
    fastfile_iw6::install::find_install(root.map(|r| r.0.as_path()), &extra)
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
    // bo2mp: Ghosts' pictures are in its zones (ui_mp), not in IWDs.
    if prefix == "iw6" {
        let install = ghosts_install(asset_transport::games_root_from_env().ok().as_ref())?;
        return fastfile_iw6::ui_image::read_ui_image(&install, names);
    }
    // bo2mp: Infinite Warfare's are in its ui.ff (and its image paks): the
    // map previews `image_mp_map_<map>`, the Zombies posters.
    if prefix == "iw7" {
        let root = asset_transport::games_root_from_env().ok()?;
        let ui = asset_transport::find_zone_file_version(&root, "ui", asset_transport::IW7_ZONE_VERSION).ok()?;
        let dir = ui.path.parent()?.to_path_buf();
        let wanted: Vec<String> = names.iter().flat_map(|n| iw7_picture_names(n)).collect();
        return fastfile_iw7::ui_image::read_ui_image(&dir, &wanted);
    }
    // bo2mp: Black Ops III's weapon icons are in its core_common (and its
    // `.xpak` packages).
    if prefix == "t7" {
        if !names.iter().any(|n| n.starts_with("t7_icon_weapon_")) {
            return None;
        }
        let icons = bo3_icons()?;
        return names.iter().find_map(|n| icons.get(n.as_str()).cloned());
    }
    // bo2mp: Advanced Warfare's are in its common_mp (and its image paks).
    if prefix == "s1" {
        let root = asset_transport::games_root_from_env().ok();
        let extra: Vec<std::path::PathBuf> = asset_transport::game_paths::get("aw").into_iter().collect();
        let install = fastfile_s1::install::find_install(root.as_ref().map(|r| r.0.as_path()), &extra)?;
        return fastfile_s1::ui_image::read_ui_image(&install, names);
    }
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

/// bo2mp shell: the names of every image a game's own IWDs hold that start
/// with `starts` (lower case, sorted; empty when the game is not installed).
/// Slow the first time (the archives are indexed): call it off the main
/// thread.
pub(crate) fn game_image_names(prefix: &str, starts: &str) -> Vec<String> {
    let Some(root) = asset_transport::games_root_from_env().ok() else { return Vec::new() };
    let trees = asset_transport::NamespaceTrees::discover(&root);
    let Some(main) = namespace_of(prefix).and_then(|ns| trees.main_for(ns)).map(std::path::Path::to_path_buf) else {
        return Vec::new();
    };
    let Ok(index) = asset_transport::IwdIndex::open(&main) else { return Vec::new() };
    let mut out: Vec<String> = index.image_names().filter(|n| n.starts_with(starts)).map(str::to_owned).collect();
    out.sort();
    out
}

/// bo2mp: Infinite Warfare's pictures for a name the menus ask for
/// (`loadscreen_<map>`, `preview_<map>`): its map preview (named for the
/// map's code name, mostly the zone name) or its Zombies poster.
fn iw7_picture_names(name: &str) -> Vec<String> {
    let map = name
        .strip_prefix("loadscreen_")
        .or_else(|| name.strip_prefix("preview_"))
        .or_else(|| name.strip_suffix("_load"))
        .unwrap_or(name);
    if let Some(cp) = map.strip_prefix("cp_") {
        return vec![format!("zm_theater_poster_{cp}")];
    }
    let code = map.strip_prefix("mp_").unwrap_or(map);
    // (ui.ff names the pictures by the maps' titles where the zone names
    // differ: Mayday is mp_breakneck, Breakout mp_parkour.)
    let code = match code {
        "breakneck" => "mayday",
        "parkour" => "breakout",
        "codphish" => "altitude",
        "rally" | "carnage2" => "carnage",
        "dome_iw" => "dome",
        "desert" => "grounded",
        "hawkwar" => "heartland",
        "divide" => "scorch",
        "rivet" => "skydock",
        "skyway" => "terminus",
        "renaissance2" => "geneva",
        "permafrost2" => "overflow",
        "turista2" => "marsoasis",
        other => other,
    };
    vec![format!("image_mp_map_{code}"), format!("image_mp_map_{code}_vote")]
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
/// loading screen, else its preview). All of a map's names share one read,
/// which `warm_map_pictures` has usually done already.
pub(crate) fn map_picture(material: &str) -> Option<std::sync::Arc<bevy::prelude::Image>> {
    let ui_name = material
        .strip_prefix("menu_")
        .and_then(|m| m.strip_suffix("_map_select_final"))
        .or_else(|| material.strip_prefix("loadscreen_"))
        .or_else(|| material.strip_prefix("menu_mp_map_select_").and_then(|m| m.strip_suffix("_big")))?;
    let map = categories().iter().flat_map(|c| c.maps.iter()).find(|m| m.ui_name == ui_name)?.clone();
    picture_async(&map_key(&map), true, move || read_map_picture(&map))
}

fn map_key(map: &ForeignMap) -> String {
    format!("map:{}", map.ui_name)
}

/// Where a map's picture is kept once read (the games' archives are slow to
/// search; this is read in a few milliseconds).
fn map_picture_cache(map: &ForeignMap) -> std::path::PathBuf {
    std::path::Path::new("iw4l-artifacts/cache/map-pictures").join(format!("{}_{}.png", map.game, map.stem))
}

/// A map's picture: from the cache, else from its game (then cached), made
/// bigger off the main thread.
fn read_map_picture(map: &ForeignMap) -> Option<(u32, u32, Vec<u8>)> {
    let cache = map_picture_cache(map);
    let small = image::open(&cache).ok().map(|i| i.to_rgba8()).map(|i| (i.width(), i.height(), i.into_raw())).or_else(|| {
        let found = read_game_image(
            map.game,
            &[format!("loadscreen_{}", map.stem), format!("preview_{}", map.stem), format!("{}_load", map.stem)],
        )?;
        if let Some(img) = image::RgbaImage::from_raw(found.0, found.1, found.2.clone()) {
            let _ = std::fs::create_dir_all(cache.parent()?);
            if let Err(e) = img.save(&cache) {
                diag::warn!(Ui, "map picture cache: {}: {e}", cache.display());
            }
        }
        Some(found)
    })?;
    Some(upscale_map_shot(small))
}

/// A small map shot (256-1023 px wide, wider than tall) made about 1600 px
/// wide with a Lanczos filter and a light unsharp mask, so it stays clean
/// full-screen and in the big map card.
pub(crate) fn upscale_map_shot((w, h, rgba): (u32, u32, Vec<u8>)) -> (u32, u32, Vec<u8>) {
    if !(128..1024).contains(&w) || h >= 1024 || (w as f32) < h as f32 * 1.3 {
        return (w, h, rgba);
    }
    let Some(buf) = image::RgbaImage::from_raw(w, h, rgba.clone()) else {
        return (w, h, rgba);
    };
    let scale = (1600.0 / w as f32).clamp(1.0, 4.0);
    let (bw, bh) = ((w as f32 * scale) as u32, (h as f32 * scale) as u32);
    let big = image::imageops::resize(&buf, bw, bh, image::imageops::FilterType::Lanczos3);
    let sharp = image::imageops::unsharpen(&big, 1.2, 4);
    (bw, bh, sharp.into_raw())
}

/// Reads every foreign map's picture in the background, one after another,
/// so the map picker's tiles are in by the time it opens (once a run).
pub(crate) fn warm_map_pictures() {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("ui-map-pictures".into())
        .spawn(|| {
            let started = std::time::Instant::now();
            let maps: Vec<ForeignMap> = categories().iter().flat_map(|c| c.maps.iter().cloned()).collect();
            let (mut found, mut cached) = (0, 0);
            for map in &maps {
                let key = map_key(map);
                if map_picture_cache(map).is_file() {
                    cached += 1;
                }
                if !claim_picture(&key) {
                    continue;
                }
                let image = read_map_picture(map).map(|p| picture_image(p, true));
                found += usize::from(image.is_some());
                store_picture(key, image);
            }
            diag::info!(
                Ui,
                "bo2mp shell: map pictures warmed: {found} read of {} maps ({cached} from the cache) in {:.1}s",
                maps.len(),
                started.elapsed().as_secs_f32()
            );
        })
        .ok();
}
