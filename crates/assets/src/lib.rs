mod artifact_cache;
mod asset_graph;
mod gltf_export;
mod iwd;
mod lane;
mod lane_capability;
pub mod loading_screen;
mod map_load_process;
// bo2mp `iw4l check-maps`: every foreign map read as a match reads it.
mod map_check;
pub use map_check::{check_map, trace_bench};
mod map_scripts;
mod match_load;
pub mod plugin;
pub mod prepared;
pub mod session_load;
mod teardown;

pub use artifact_cache::{cache_flight, cache_get, cache_put, fnv1a64, fnv1a64_more};
pub(crate) use asset_graph::stamp_match_destructible_death;
pub use asset_graph::{
    AssetEdge, AssetEdgeCensus, AssetEdgeReason, AssetGraphCensus, DESTRUCTIBLE_DEATH_HINTS,
    DeathClipEdge, DeathHuskEdge, DestructibleDeathHint, DestructibleDeathRow, FpvMeshIndex,
    FpvMeshSpace, FxIndex, FxModelIndex, FxModelSpace, FxSpace, LoadedSoundIndex, LoadedSoundSpace,
    MapXModelIndex, MapXModelSpace, MaterialIndex, MaterialSpace, ProjectileModelIndex,
    ProjectileModelSpace, SoundAliasIndex, SoundAliasSpace, TechniqueSetIndex, TechniqueSetSpace,
    TracerIndex, TracerSpace, WorldWeaponIndex, WorldWeaponSpace, XAnimIndex, XAnimSpace,
    resolve_after_absorb,
};
pub use fastfile_iw4::GlyphCapture;
pub use gltf_export::{GltfExportSummary, export_prepared_world_gltf};
pub use iwd::{IwdSoundIndex, NamespaceSoundIwd, NamespaceTree, NamespaceTrees};
pub use lane::{check_iw4_map, CommonCensus, LANE_GAPS, LaneGap, LoadedWorld, ZoneLane, lane};
// bo2mp lane C: the multiplayer front end (no map).
pub use lane::t6_frontend::{T6Frontend, load_t6_frontend, load_t6_zm_pictures, plain_rgba8};
// bo2zm M4: the Zombies front end (no map), a Resource kept for the run;
// renamed here so it sits beside the multiplayer front end.
pub use lane::{T6Frontend as T6ZmFrontend, load_t6_frontend as load_t6_zm_frontend};
/// bo2mp: the mod's Create-a-Class pictures (`IW4L_T6_MOD`).
pub use lane::mod_picture_rgba as t6_mod_picture_rgba;
pub use lane_capability::{LaneStatus, PreparedCapability, lane_status};
pub use lighting_iw4::MODEL_LIGHTING_TILE_BYTES;
pub use loading_screen::{LoadingPreviewSource, LoadingScreen};
pub use map_load_process::MapLoadProcess;
pub use match_load::{
    MapLoadApproval, MatchLoadAbort, MatchLoadAccepted, MatchLoadBusy, MatchLoadDispatch,
    MatchLoadRequest, PreparedMatchReady, PreparedMatchSound,
};
pub use plugin::AssetPlugin;
pub use prepared::{
    MapFacts, MatchMaterials, MatchType10SoundHints, PreparedBodies, PreparedBodyClips,
    PreparedDestructibleDeath, PreparedFpvMeshes, PreparedGaps, PreparedLocalizedStrings,
    PreparedMap, PreparedProjectileMeshes, PreparedWeapons, PreparedWorldWeapons, PreparedXAnims,
    PreparedXModelWalkCensus, SessionCompass, T6HudFont, T6HudFonts, T6HudIcons, T6ScreenFilter,
    T6ScreenFilters, T6Ui,
};
pub use session_load::{
    check_guest_guns, MatchLoadOutcome, MatchMaterialSeed, PreparedMatch, PreparedWorld, ShellCommon,
    apply_match_material_map, load_match_material_catalog, load_match_material_seed, load_pool,
    load_prepared_match, load_shell_common, load_workers, note_gpu_resident, publish_process_cpus,
    release_resident_cpu_images,
};

pub mod image_handles;

/// bo2mp: a match the Black Ops II front end starts plays by Black Ops II's
/// rules on any map: a Modern Warfare 2, Modern Warfare 3 or Black Ops map
/// brings its world (drawing, collision, entities, ambience) and Black Ops
/// II brings the rest (guns, scripts, HUD, factions, sound).
static T6_RULES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn set_t6_rules(on: bool) {
    T6_RULES.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn t6_rules() -> bool {
    T6_RULES.load(std::sync::atomic::Ordering::Relaxed)
}

/// bo2mp: which game the loaded match's map comes from (its own namespace:
/// Iw4 MW2, Iw5 MW3, T5 Black Ops, T6 Black Ops II, Iw3 CoD4, T4 World at
/// War), set by the match load; the players' bodies follow it.
static MAP_GAME: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

const MAP_GAMES: [asset_core::AssetNamespace; 7] = [
    asset_core::AssetNamespace::Iw4,
    asset_core::AssetNamespace::T5,
    asset_core::AssetNamespace::Iw5,
    asset_core::AssetNamespace::T6,
    asset_core::AssetNamespace::Iw6,
    asset_core::AssetNamespace::T4,
    asset_core::AssetNamespace::Iw3,
];

pub fn set_map_game(game: Option<asset_core::AssetNamespace>) {
    let code = game.and_then(|g| MAP_GAMES.iter().position(|m| *m == g)).map_or(0, |i| i as u8 + 1);
    MAP_GAME.store(code, std::sync::atomic::Ordering::Relaxed);
}

/// bo2mp: Graphics > Player Models: another game's map shows its own
/// soldiers (true) or Black Ops II's characters.
static MAP_GAME_BODIES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_map_game_bodies(on: bool) {
    MAP_GAME_BODIES.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn map_game_bodies() -> bool {
    MAP_GAME_BODIES.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn map_game() -> Option<asset_core::AssetNamespace> {
    let code = MAP_GAME.load(std::sync::atomic::Ordering::Relaxed);
    MAP_GAMES.get(usize::from(code).checked_sub(1)?).copied()
}

/// bo2mp: other games' guns join Black Ops II's as extra weapons (off with
/// `BO2MP_GUEST_GUNS=0`).
pub fn guest_guns() -> bool {
    std::env::var("BO2MP_GUEST_GUNS").map_or(true, |v| v.trim() != "0")
}

mod script_sources;
pub use script_sources::{ScriptSources, ScriptTable, T6AnimFacts, T6ScriptSet};
