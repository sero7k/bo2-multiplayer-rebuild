mod helpers;
mod iw4;
pub use iw4::check_iw4_map;
mod iw5;
// bo2mp: Call of Duty 4 maps (no ZoneGame of their own).
mod bo2_look;
mod cod4;
// bo2mp: World at War maps, through the CoD4 lane's builders.
mod waw;
// bo2mp: Ghosts' guns as guest guns.
mod iw6;
// bo2mp: Ghosts' multiplayer maps (no ZoneGame of their own).
mod iw6_map;
pub(crate) use iw6_map::load_iw6_world;
// bo2mp: Infinite Warfare's maps (no ZoneGame of their own).
mod iw7;
pub(crate) use iw7::load_iw7_world;
// bo2mp: Black Ops III's maps (no ZoneGame of their own).
mod t7_look;
mod t7_map;
pub(crate) use t7_map::load_t7_world;
// bo2mp: Infinite Warfare's Zombies maps as Black Ops II Zombies.
mod iw7_zombies;
#[allow(unused_imports)] // (register_foreign_zm_map: for the Black Ops III lane)
pub(crate) use iw7_zombies::{attach_t6_zm_rules, is_iw7_zombies_map, register_foreign_zm_map};
pub(crate) use iw6::{
    Iw6Guests, build_iw6_guest_guns, iw6_anim_check, iw6_sound_check, prefetch_iw6_guest_guns, walk_iw6_guest_guns,
};
// bo2mp: Advanced Warfare's guns as guest guns.
mod s1;
pub(crate) use s1::{S1Guests, build_s1_guest_guns, prefetch_s1_guest_guns, s1_anim_check, walk_s1_guest_guns};
// bo2mp: Call of Duty 4's and World at War's guns as guest guns.
mod legacy_guns;
// bo2mp: Black Ops III's guns as guest guns.
mod t7;
mod t7_fields;
pub(crate) use t7::{T7Guests, build_t7_guest_guns, prefetch_t7_guest_guns, t7_anim_check, t7_sound_check, walk_t7_guest_guns};
pub(crate) use legacy_guns::{
    Legacy, LegacyGuests, build_legacy_guest_guns, legacy_common_mp, load_legacy_guests, prefetch_legacy_guest_guns,
};
mod sink;
mod t5;
// bo2zm: Black Ops II.
mod t6;
mod t6_materials;
// bo2zm M2: guns, effects and sound.
mod t6_m2;
// bo2mp lane C: the multiplayer front end's UI (no map).
pub mod t6_frontend;
mod t6_scripts;

// bo2zm M4: Black Ops II's front end (no map).
pub use t6_m2::{T6Frontend, load_t6_frontend};
pub use t6_materials::mod_picture_rgba;

use std::path::Path;

pub(crate) use helpers::{
    build_iw5_static_model_draw, build_static_model_draw, build_t5_static_model_draw,
};
pub(crate) use sink::{CommonWalkSink, MaterialPopulationSink, ZoneWalkSink};
pub(crate) use t6::{attach_t6_rules, t6_surface_list};
pub(crate) use cod4::load_iw3_world;
pub(crate) use waw::load_t4_world;

use crate::{
    lane_capability::{LaneStatus, PreparedCapability},
    session_load::{PreparedWorld, WorldDrawPolicy},
};
use asset_anim::XAnimBuild;
use asset_core::ZoneGame;
use asset_game::WeaponBuild;
use asset_model::{BodyMeshBuild, FpvMeshBuild, WorldWeaponBuild};
use asset_transport::{LoadProgress, ZoneImage};

#[derive(Clone, Debug)]
pub struct LaneGap {
    pub capability: PreparedCapability,
    pub reason: String,
    pub addr: Option<&'static str>,
}

#[derive(Default)]
pub struct LoadedWorld {
    pub scripts: crate::ScriptSources,
    pub world: PreparedWorld,
    /// What the map zone itself captured. The local material indices in
    /// `world` are indices into this pool until the match finalizes one.
    pub materials: asset_material::MaterialCatalog,
    pub collision: Option<asset_world::ClipCollision>,
    pub spawns: Vec<asset_world::SpawnPoint>,
    pub bodies: BodyMeshBuild,
    pub fpv_meshes: FpvMeshBuild,
    pub xanims: XAnimBuild,
    /// bo2zm: weapons the map zone itself defines (Black Ops II maps carry
    /// their own; other games take theirs from common zones).
    pub weapons: WeaponBuild,
    /// bo2zm: the map's own weapons' third-person models.
    pub world_weapons: WorldWeaponBuild,
    /// bo2zm: tracers the map zone itself defines.
    pub tracers: asset_game::TracerCatalog,
    /// bo2mp: the bullet penetration depths the map's zones carry (Black
    /// Ops II has no common zones read yet; its table comes this way).
    pub pen_table: Option<weapon_iw4::PenetrationDepthTable>,
    pub facts: crate::MapFacts,
    /// Bytes the zone arenas held while the walk read them. The arenas
    /// themselves die with the walk; only their size travels.
    pub arena_bytes: usize,
    pub sound: Option<Result<asset_audio::SoundCatalog, String>>,
    pub report: Vec<String>,
    pub gaps: Vec<LaneGap>,
}

impl LoadedWorld {
    pub fn with_gap(
        policy: WorldDrawPolicy,
        capability: PreparedCapability,
        reason: impl Into<String>,
        addr: Option<&'static str>,
    ) -> Self {
        let reason = reason.into();
        Self {
            world: PreparedWorld {
                policy,
                ..Default::default()
            },
            report: vec![reason.clone()],
            gaps: vec![LaneGap {
                capability,
                reason,
                addr,
            }],
            ..Default::default()
        }
    }

    pub fn push_gap(
        &mut self,
        capability: PreparedCapability,
        reason: impl Into<String>,
        addr: Option<&'static str>,
    ) {
        self.gaps.push(LaneGap {
            capability,
            reason: reason.into(),
            addr,
        });
    }
}

#[derive(Default)]
pub struct CommonCensus {
    pub scripts: crate::ScriptSources,
    pub scene_models: asset_world::MapXModelSceneCatalog,
    pub shared_surfaces: asset_model::SharedXModelSurfaces,
    pub weapons: WeaponBuild,

    pub cac_tables: Vec<asset_game::CapturedStringTable>,
    pub fpv: FpvMeshBuild,
    pub world_weapons: WorldWeaponBuild,

    pub projectile_meshes: asset_model::ProjectileMeshBuild,
    pub xanims: XAnimBuild,
    pub player_anim_sources: asset_anim::PlayerAnimSources,
    pub fx: asset_game::FxCatalog,
    pub fx_models: asset_game::FxModelCatalog,

    pub tracers: asset_game::TracerCatalog,
    pub impact_fx: Option<asset_game::OwnedFxImpactTable>,

    pub material_population: asset_material::MaterialCatalog,

    pub light_defs: Vec<asset_world::CapturedLightDef>,
    pub report: Vec<String>,

    pub pen_table: weapon_iw4::PenetrationDepthTable,
    pub pen_table_loaded: bool,
    pub lochit_table: Option<[f32; weapon_iw4::HITLOC_COUNT]>,

    pub xmodel_walk: crate::PreparedXModelWalkCensus,

    /// Bytes the common_mp arenas held while the walk read them; the arenas
    /// themselves do not outlive it.
    pub s1_common_bytes: usize,

    pub teamsets: std::collections::HashMap<String, asset_game::MapTeamSettings>,
    pub film_visions: std::collections::BTreeMap<
        String,
        Result<asset_world::FilmVision, asset_world::FilmVisionParseError>,
    >,

    pub pending_images: Option<asset_material::material_images::ImageDemandPlan>,
}

pub struct MaterialPopulation {
    pub materials: asset_material::MaterialCatalog,
    pub light_defs: Vec<asset_world::CapturedLightDef>,
    pub walked: usize,
    pub report: Vec<String>,
    pub cac_tables: Vec<asset_game::CapturedStringTable>,
    pub scripts: crate::ScriptSources,
}

impl Default for MaterialPopulation {
    fn default() -> Self {
        Self {
            materials: asset_material::MaterialCatalog::default(),
            light_defs: Vec::new(),
            walked: 0,
            report: Vec::new(),
            cac_tables: Vec::new(),
            scripts: crate::ScriptSources::default(),
        }
    }
}

pub trait ZoneLane: Send + Sync {
    fn game(&self) -> ZoneGame;
    fn capabilities(&self) -> &'static [(PreparedCapability, LaneStatus)];

    fn load_world(
        &self,
        path: &Path,
        image: &ZoneImage,
        progress: &LoadProgress,

        shared_surfaces: asset_model::SharedXModelSurfaces,

        material_seed: asset_material::MaterialCatalog,
        common_film_visions: &mut std::collections::BTreeMap<
            String,
            Result<asset_world::FilmVision, asset_world::FilmVisionParseError>,
        >,
    ) -> LoadedWorld;

    fn load_common_mp(
        &self,
        path: &Path,
        image: &ZoneImage,
        progress: &LoadProgress,
        decode_color_maps: bool,
        material_seed: asset_material::MaterialCatalog,
    ) -> CommonCensus;

    fn load_material_population(
        &self,
        path: &Path,
        image: &ZoneImage,
        progress: &LoadProgress,
        material_seed: asset_material::MaterialCatalog,
    ) -> MaterialPopulation;
}

pub fn lane(game: ZoneGame) -> &'static dyn ZoneLane {
    match game {
        ZoneGame::Iw4 => &iw4::Iw4Lane,
        ZoneGame::T5 => &t5::T5Lane,
        ZoneGame::Iw5 => &iw5::Iw5Lane,
        ZoneGame::T6 => &t6::T6Lane,
    }
}

pub const LANE_GAPS: &[&str] = &[
    "assets::lane::iw4::load_world/zone_header",
    "assets::lane::iw4::load_world/zone_arenas",
    "assets::lane::iw4::load_world/no_gfx_world",
    "assets::lane::iw4::load_world/world_mesh",
    "assets::lane::t5::load_world/zone_header",
    "assets::lane::t5::load_world/zone_arenas",
    "assets::lane::t5::load_world/no_gfx_world",
    "assets::lane::t5::load_world/world_mesh",
    "assets::lane::iw5::load_world/zone_header",
    "assets::lane::iw5::load_world/zone_arenas",
    "assets::lane::iw5::load_world/no_gfx_world",
    "assets::lane::iw5::load_world/world_mesh",
    "assets::lane::t6::load_world/walk",
    "assets::lane::t6::load_world/no_world",
    "assets::lane::t6::load_world/world_mesh",
    "assets::session_load::load_prepared_match/zone_open",
];
