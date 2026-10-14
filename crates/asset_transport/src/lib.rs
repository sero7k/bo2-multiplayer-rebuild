pub mod artifact_cache;
pub mod discover;
pub mod iwd;
pub mod load_jobs;
pub mod namespace_trees;
pub mod progress;
pub mod t6_extra;
pub mod t6_mod;
pub mod zone;

pub use artifact_cache::{
    CacheFlight, cache_flight, cache_get, cache_open, cache_put, cache_put_with, fnv1a64,
    fnv1a64_more,
};
pub use asset_core::ZoneGame;
pub mod game_paths;
pub use discover::{
    GamesRoot, MapPack, ZoneFile, ensure_artifacts_dir, find_common_mp_for_envelope,
    find_common_mp_for_zone, find_localized_common_mp_for_zone, find_runtime_common_mp,
    find_runtime_zone, find_zone_file, find_zone_file_version, find_zone_for_tree,
    game_root_for_zone, games_content_report, games_root_from_env, games_root_report,
    group_mp_maps, list_mp_map_packs, list_mp_maps, load_dotenv, map_load_title, peek_zone_version,
    split_zone_key, zone_game_for_path, zone_version,
};
pub use discover::{
    IW4X_MAGIC, USERMAPS_PACK, Usermap, is_iw4_variant_version, is_iw4x_envelope, list_usermaps,
    usermap_iwds_for_main, usermap_zone_of,
};
pub use discover::{
    IW3_ZONE_VERSION, IW6_ZONE_VERSION, IW7_ZONE_VERSION, T4_ZONE_VERSION, T7_ZONE_VERSION, is_iw7_map, is_t7_map, list_t7_zombies_maps, legacy_game_for_path, legacy_version, list_iw7_zombies_maps, list_legacy_mp_map_files, list_legacy_mp_maps,
};
pub use iwd::{
    IwdFile, IwdIndex, IwdSoundIndex, cached_iwd_dirs, game_main_for_zone, game_mains_under,
    inflate_zlib, iwd_entry_reads, iwd_read_cost, read_iwd_named, read_main_iwd_file, read_text,
};
pub use load_jobs::{CacheResult, Job, JobKind};
pub use namespace_trees::{NamespaceSoundIwd, NamespaceTree, NamespaceTrees};
pub use progress::{
    LoadLaneTiming, LoadProgress, LoadSnapshot, StageEnd, StageHandle, StageId, StageKey,
    StageOutcome, StageScope, StageSnapshot, WorkCount, peak_resident_bytes,
    process_resident_bytes,
};
pub use zone::{
    Iw4WireFormat, Iw5ZoneMemory, T5ZoneMemory, T6ZoneMemory, ZoneImage, ZoneMemory, ZoneOpenError,
    open_zone, open_zone_shared, parse_zone_image, xfile_arena_row, zone_share_counts,
};
