use std::path::Path;
use std::sync::Arc;

use fastfile_iw4::{
    AssetLinkSink, AssetSink, AssetType, Ptr, ScriptStrings, ZoneStream, load_asset_at_observed,
    load_zone,
};

use crate::discover::{GamesRoot, find_runtime_zone, find_zone_file, find_zone_file_version};
use asset_transport::discover::find_t5_localized_zone;
use crate::sound_catalog::SoundCatalog;
use crate::zone::{ZoneMemory, open_zone_shared};
use crate::zone_sound::{ZoneSoundOrigin, ensure_zone_sound};
use crate::{AssetNamespace, ZoneGame};

pub fn namespace_for_zone(games: &GamesRoot, zone: &str) -> AssetNamespace {
    find_zone_file(games, zone)
        .ok()
        .and_then(|found| crate::zone_game_for_path(&found.path))
        .map(AssetNamespace::from_zone_game)
        .unwrap_or(AssetNamespace::Iw4)
}

struct SoundSink {
    catalog: SoundCatalog,
    walked: usize,
    stopped_at: Option<(usize, &'static str)>,
}

impl AssetSink for SoundSink {
    fn set_script_strings(&mut self, _strings: ScriptStrings) {}

    fn load_asset(
        &mut self,
        s: &mut ZoneStream<'_>,
        index: usize,
        ty: AssetType,
        slot: Ptr,
    ) -> fastfile_iw4::Result<()> {
        self.stopped_at = Some((index, ty.name()));
        load_asset_at_observed(s, ty, slot, self)?;
        self.walked += 1;
        self.stopped_at = None;
        Ok(())
    }
}

impl AssetLinkSink for SoundSink {
    fn loaded(
        &mut self,
        s: &ZoneStream<'_>,
        ty: AssetType,
        slot: Ptr,
        insert_slot: Option<Ptr>,
    ) -> fastfile_iw4::Result<()> {
        self.catalog.loaded(s, ty, slot, insert_slot)
    }

    fn alias(&mut self, ty: AssetType, slot: Ptr, target: Ptr) -> fastfile_iw4::Result<()> {
        self.catalog.alias(ty, slot, target)
    }

    fn capture_loaded_sound(
        &mut self,
        s: &ZoneStream<'_>,
        header: Ptr,
        pcm: Ptr,
        data_len: usize,
    ) -> fastfile_iw4::Result<()> {
        self.catalog.capture_loaded_sound(s, header, pcm, data_len)
    }

    fn bind_last_loaded_to_sound_file(&mut self, file: Ptr) -> fastfile_iw4::Result<()> {
        self.catalog.bind_last_loaded_to_sound_file(file)
    }

    fn bind_streamed_sound_file(
        &mut self,
        file: Ptr,
        dir: &str,
        name: &str,
    ) -> fastfile_iw4::Result<()> {
        self.catalog.bind_streamed_sound_file(file, dir, name)
    }

    fn capture_raw_file(
        &mut self,
        name: &str,
        data: &[u8],
        zlib_compressed: bool,
    ) -> fastfile_iw4::Result<()> {
        self.catalog.ingest_rawfile(name, data, zlib_compressed);
        Ok(())
    }

    fn capture_sound(
        &mut self,
        s: &ZoneStream<'_>,
        list: Ptr,
        count: usize,
        head: Option<Ptr>,
    ) -> fastfile_iw4::Result<()> {
        self.catalog.capture_sound(s, list, count, head)
    }

    fn capture_snd_curve(&mut self, s: &ZoneStream<'_>, header: Ptr) -> fastfile_iw4::Result<()> {
        self.catalog.capture_snd_curve(s, header)
    }
}

pub fn load_sound_catalog(path: &Path) -> Result<SoundCatalog, String> {
    let image = open_zone_shared(path).map_err(|e| e.to_string())?;
    let header = image.header().map_err(|e| e.to_string())?;
    let mut memory = ZoneMemory::for_header(&header);
    let mut stream = memory.stream(&image.bytes).map_err(|e| e.to_string())?;
    let mut sink = SoundSink {
        catalog: SoundCatalog::default(),
        walked: 0,
        stopped_at: None,
    };
    sink.catalog
        .set_capture_zone(crate::ZoneOwner::from_zone_path(path));
    sink.catalog.set_capture_game(crate::ZoneGame::Iw4);
    load_zone(&mut stream, &mut sink).map_err(|e| {
        format!(
            "sound catalog failed after {} assets at {:?}: {e}",
            sink.walked, sink.stopped_at
        )
    })?;
    sink.catalog.resolve_curve_knots();
    sink.catalog.resolve_ent_channels();
    sink.catalog.publish();
    Ok(sink.catalog)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundZoneGap {
    pub namespace: AssetNamespace,

    pub zone: String,
    pub reason: String,
}

#[derive(Default)]
pub struct LoadedSoundBank {
    pub catalog: SoundCatalog,
    pub gaps: Vec<SoundZoneGap>,
}

fn namespace_of_game(game: ZoneGame) -> AssetNamespace {
    AssetNamespace::from_zone_game(game)
}

pub(crate) fn walk_zone_sound(path: &Path) -> Result<SoundCatalog, String> {
    let game = crate::zone_game_for_path(path)
        .ok_or_else(|| format!("{} is not a readable IWff envelope", path.display()))?;
    match game {
        ZoneGame::Iw4 => load_sound_catalog(path),
        ZoneGame::Iw5 => crate::sound_load_iw5::load_sound_catalog_iw5(path),
        ZoneGame::T5 => crate::sound_load_t5::load_sound_catalog_t5(path),
        ZoneGame::T6 => Err(format!(
            "{}: Black Ops II sound banks are not read yet (bo2zm)",
            path.display()
        )),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Merge {
    Override,
    Missing,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Absent {
    Skip,
    Gap,
}

struct SoundSource {
    namespace: AssetNamespace,
    name: &'static str,
    found: Result<std::path::PathBuf, String>,
    merge: Merge,
    absent: Absent,
}

fn sound_sources(games: &GamesRoot, map: &Path) -> (Vec<SoundSource>, Vec<SoundSource>) {
    let runtime = |name: &'static str, merge, absent| SoundSource {
        namespace: AssetNamespace::Iw4,
        name,
        found: find_runtime_zone(games, map, name).map(|zone| zone.path),
        merge,
        absent,
    };
    let donor = |namespace, name: &'static str, version| SoundSource {
        namespace,
        name,
        found: find_zone_file_version(games, name, version).map(|zone| zone.path),
        merge: Merge::Missing,
        absent: Absent::Skip,
    };
    let before_map = vec![
        runtime("code_post_gfx_mp", Merge::Override, Absent::Skip),
        runtime("localized_code_post_gfx_mp", Merge::Override, Absent::Skip),
        runtime("patch_mp", Merge::Override, Absent::Skip),
        runtime("common_mp", Merge::Override, Absent::Gap),
        runtime("localized_common_mp", Merge::Override, Absent::Gap),
    ];
    // bo2mp: MW2's guns as extra weapons in a Black Ops II match: its
    // common sounds too, when the map did not already bring MW2's common.
    let runtime_common = before_map
        .iter()
        .find(|s| s.name == "common_mp")
        .and_then(|s| s.found.as_ref().ok().cloned());
    let mw2_guest = |name: &'static str| {
        let found = find_zone_file_version(games, name, fastfile_iw4::ZONE_VERSION_PC).map(|zone| zone.path);
        let already = found.as_ref().ok().zip(runtime_common.as_ref()).is_some_and(|(f, r)| f == r)
            || (name == "localized_common_mp" && runtime_common.as_ref().is_some_and(|r| {
                found.as_ref().is_ok_and(|f| f.parent() == r.parent())
            }));
        SoundSource {
            namespace: AssetNamespace::Iw4,
            name,
            found: if already { Err("MW2's common sounds are already the runtime's".to_owned()) } else { found },
            merge: Merge::Missing,
            absent: Absent::Skip,
        }
    };
    let mut after_map = vec![mw2_guest("common_mp"), mw2_guest("localized_common_mp")];
    after_map.extend([
        donor(
            AssetNamespace::Iw5,
            "common_mp",
            fastfile_iw5::ZONE_VERSION_PC,
        ),
        donor(
            AssetNamespace::Iw5,
            "localized_common_mp",
            fastfile_iw5::ZONE_VERSION_PC,
        ),
        donor(
            AssetNamespace::T5,
            "code_post_gfx_mp",
            fastfile_t5::ZONE_VERSION_PC,
        ),
        donor(
            AssetNamespace::T5,
            "common_mp",
            fastfile_t5::ZONE_VERSION_PC,
        ),
        donor(
            AssetNamespace::T5,
            "localized_common_mp",
            fastfile_t5::ZONE_VERSION_PC,
        ),
        bo1_announcer_source(games),
        mw3_voice_source("localized_common_mp", "localized_common_mp (MW3 announcer)"),
        mw3_voice_source("localized_code_post_gfx_mp", "localized_code_post_gfx_mp (MW3 UI sounds)"),
    ]);
    // bo2mp: Ghosts' guns as extra weapons: their sounds, from its zones
    // (read by `fastfile_iw6`, kept under the Ghosts namespace).
    if std::env::var("BO2MP_GUEST_GUNS").map_or(true, |v| v.trim() != "0") {
        after_map.extend(ghosts_sound_sources(games));
        // bo2mp: World at War's guns: their sounds, from its common zone
        // (read by `t4`, kept under the World at War namespace). Nothing
        // when World at War is not installed.
        after_map.extend(waw_sound_source(games));
        // bo2mp: Advanced Warfare's guns: their sounds, from its common_mp
        // (read by `fastfile_s1`, kept under the AW namespace).
        after_map.extend(aw_sound_source(games));
        // bo2mp: Black Ops III's guns: their sounds, from its mp_common and
        // its sound banks (read by `fastfile_t7`, kept under the BO3
        // namespace).
        after_map.extend(bo3_sound_source(games));
    }
    // bo2mp: Resurgence's sounds from Modern Warfare III (2023), when the
    // match is Resurgence and the install is set.
    after_map.extend(jup_sound_source());
    (before_map, after_map)
}

/// bo2mp: Modern Warfare III's (2023) install, for Resurgence's sounds.
fn jup_sound_source() -> Option<SoundSource> {
    crate::jup_sounds_wanted()?;
    Some(SoundSource {
        namespace: AssetNamespace::Jup,
        name: "Resurgence (MW3 2023)",
        found: crate::jup_install().ok_or_else(|| {
            format!("{} is not set to a Modern Warfare III (2023) install", crate::JUP_INSTALL_ENV)
        }),
        merge: Merge::Missing,
        absent: Absent::Skip,
    })
}

/// bo2mp: World at War's common zone, its guns' sounds, when it is
/// installed.
fn waw_sound_source(games: &GamesRoot) -> Option<SoundSource> {
    Some(SoundSource {
        namespace: AssetNamespace::T4,
        name: "common_mp (World at War guns)",
        found: Ok(crate::waw_common_mp(games)?),
        merge: Merge::Missing,
        absent: Absent::Skip,
    })
}

/// bo2mp: Advanced Warfare's common_mp, its guns' sounds, when AW is
/// installed.
fn aw_sound_source(games: &GamesRoot) -> Option<SoundSource> {
    let path = crate::aw_install(Some(games.0.as_path()))?.join("zone").join("common_mp.ff");
    Some(SoundSource {
        namespace: AssetNamespace::S1,
        name: "common_mp (AW guns)",
        found: Ok(path),
        merge: Merge::Missing,
        absent: Absent::Skip,
    })
}

/// bo2mp: Black Ops III's mp_common, its guns' sounds, when it is
/// installed.
fn bo3_sound_source(games: &GamesRoot) -> Option<SoundSource> {
    Some(SoundSource {
        namespace: AssetNamespace::T7,
        name: "mp_common (BO3 guns)",
        found: Ok(crate::bo3_mp_common(games)?),
        merge: Merge::Missing,
        absent: Absent::Skip,
    })
}

/// bo2mp: the Ghosts zones its guns' sounds come from, when Ghosts is
/// installed.
fn ghosts_sound_sources(games: &GamesRoot) -> Vec<SoundSource> {
    let Some(install) = crate::ghosts_install(Some(games.0.as_path())) else {
        return Vec::new();
    };
    crate::ghosts_sound_zones(&install)
        .into_iter()
        .zip(["common_mp (Ghosts guns)", "common_core_mp (Ghosts guns)"])
        .map(|(path, name)| SoundSource {
            namespace: AssetNamespace::Iw6,
            name,
            found: Ok(path),
            merge: Merge::Missing,
            absent: Absent::Skip,
        })
        .collect()
}

/// BO1's multiplayer announcer (`vox_ops_*`, `vox_rus_*`) is carried by its
/// maps' language zones, not by `common_mp`. Nuketown's holds both the Black
/// Ops and Spetsnaz voices, which the BO1 presentation speaks with.
pub const BO1_ANNOUNCER_ZONE: &str = "mp_nuked";

/// `IW4L_VOICE_GAMES`: a folder of games lent for their announcer only (MW3,
/// for lines BO1 never had). Nothing else is read from it, so a match does
/// not take in that game's weapons or materials.
pub fn voice_games_root() -> Option<GamesRoot> {
    std::env::var_os("IW4L_VOICE_GAMES")
        .filter(|root| !root.is_empty())
        .map(|root| GamesRoot(std::path::PathBuf::from(root)))
}

/// An MW3 zone from the voice-only folder: its announcer
/// (`localized_common_mp`) or its interface sounds
/// (`localized_code_post_gfx_mp`).
fn mw3_voice_source(zone: &'static str, name: &'static str) -> SoundSource {
    let found = voice_games_root()
        .ok_or_else(|| "IW4L_VOICE_GAMES is not set".to_owned())
        .and_then(|root| find_zone_file_version(&root, zone, fastfile_iw5::ZONE_VERSION_PC))
        .map(|zone| zone.path);
    SoundSource {
        namespace: AssetNamespace::Iw5,
        name,
        found,
        merge: Merge::Missing,
        absent: Absent::Skip,
    }
}

fn bo1_announcer_source(games: &GamesRoot) -> SoundSource {
    let found = find_zone_file_version(games, "common_mp", fastfile_t5::ZONE_VERSION_PC)
        .and_then(|common| find_t5_localized_zone(&common.path, Some("english"), BO1_ANNOUNCER_ZONE))
        .and_then(|zone| {
            zone.map(|zone| zone.path)
                .ok_or_else(|| format!("no language zone for {BO1_ANNOUNCER_ZONE}"))
        });
    SoundSource {
        namespace: AssetNamespace::T5,
        name: "localized_mp_nuked",
        found,
        merge: Merge::Missing,
        absent: Absent::Skip,
    }
}

pub struct SoundSources {
    before_map: SoundCatalog,
    after_map: Vec<(SoundSource, Arc<SoundCatalog>)>,
    gaps: Vec<SoundZoneGap>,
}

fn origin_label(origin: ZoneSoundOrigin) -> String {
    match origin {
        ZoneSoundOrigin::Shared(walk) => format!("{walk} walk"),
        ZoneSoundOrigin::AudioOnly => "audio-only walk".to_owned(),
    }
}

pub fn gather_sound_sources(games: &GamesRoot, map: &Path) -> SoundSources {
    let (before, after) = sound_sources(games, map);
    let mut sources = SoundSources {
        before_map: SoundCatalog::default(),
        after_map: Vec::new(),
        gaps: Vec::new(),
    };
    for source in before {
        if let Some(catalog) = sources.take(&source) {
            sources.before_map.absorb_unresolved((*catalog).clone());
        }
    }
    for source in after {
        if let Some(catalog) = sources.take(&source) {
            sources.after_map.push((source, catalog));
        }
    }
    sources
}

impl SoundSources {
    fn take(&mut self, source: &SoundSource) -> Option<Arc<SoundCatalog>> {
        let path = match &source.found {
            Ok(path) => path,
            Err(error) => {
                match source.absent {
                    Absent::Skip => diag::info!(
                        Zone,
                        "sound bank: {} {} missing ({error})",
                        source.namespace.as_str(),
                        source.name
                    ),
                    Absent::Gap => self.gap(source.namespace, source.name, error.clone()),
                }
                return None;
            }
        };
        // bo2mp: Ghosts zones are read by `fastfile_iw6`, World at War's by
        // `t4` (neither has a `ZoneGame`).
        let (catalog, origin) = match source.namespace {
            AssetNamespace::Iw6 => (crate::ghosts_weapon_sounds(path), ZoneSoundOrigin::AudioOnly),
            AssetNamespace::T4 => (crate::waw_weapon_sounds(path), ZoneSoundOrigin::AudioOnly),
            AssetNamespace::S1 => (crate::aw_weapon_sounds(path), ZoneSoundOrigin::AudioOnly),
            AssetNamespace::T7 => (crate::bo3_weapon_sounds(path), ZoneSoundOrigin::AudioOnly),
            AssetNamespace::Jup => (
                crate::jup_resurgence_sounds(path, crate::jup_sounds_wanted().unwrap_or(false)),
                ZoneSoundOrigin::AudioOnly,
            ),
            _ => ensure_zone_sound(path),
        };
        match catalog {
            Ok(catalog) => {
                diag::info!(
                    Zone,
                    "sound bank: {} {} from the {} — {} aliases {} loaded",
                    source.namespace.as_str(),
                    source.name,
                    origin_label(origin),
                    catalog.sounds.len(),
                    catalog.loaded.len()
                );
                Some(catalog)
            }
            Err(error) => {
                self.gap(source.namespace, source.name, error);
                None
            }
        }
    }

    fn gap(&mut self, namespace: AssetNamespace, zone: &str, reason: String) {
        self.gaps.push(SoundZoneGap {
            namespace,
            zone: zone.to_owned(),
            reason,
        });
    }
}

pub fn compose_sound_bank(
    sources: SoundSources,
    zone: &str,
    namespace: AssetNamespace,
    map: Result<SoundCatalog, String>,
) -> LoadedSoundBank {
    let SoundSources {
        before_map,
        after_map,
        gaps,
    } = sources;
    let mut bank = LoadedSoundBank {
        catalog: before_map,
        gaps,
    };
    match map {
        Ok(map) => {
            diag::info!(
                Zone,
                "sound bank: map zone {zone} ({}) — {} rawfiles, {} aliases",
                namespace.as_str(),
                map.rawfiles.len(),
                map.sounds.len()
            );
            match namespace {
                AssetNamespace::Iw4 => bank.catalog.absorb_unresolved(map),
                _ => bank.catalog.absorb_missing_aliases_unresolved(map),
            }
        }
        Err(error) => bank.gap(namespace, zone, error),
    }
    for (source, catalog) in after_map {
        let catalog = (*catalog).clone();
        match source.merge {
            Merge::Override => bank.catalog.absorb_unresolved(catalog),
            Merge::Missing => bank.catalog.absorb_missing_aliases_unresolved(catalog),
        }
    }
    bank.catalog.finalize();
    for gap in &bank.gaps {
        diag::warn!(
            Zone,
            "sound bank gap: {} `{}` — {}",
            gap.namespace.as_str(),
            gap.zone,
            gap.reason
        );
    }
    bank
}

pub fn load_mp_sound_bank(games: &GamesRoot, zone: &str) -> Result<LoadedSoundBank, String> {
    let zone_ff = find_zone_file(games, zone)?;
    let sources = gather_sound_sources(games, &zone_ff.path);
    let namespace =
        crate::zone_game_for_path(&zone_ff.path).map_or(AssetNamespace::Iw4, namespace_of_game);
    let (map, _) = ensure_zone_sound(&zone_ff.path);
    let map = map.map(|catalog| (*catalog).clone());
    Ok(compose_sound_bank(sources, zone, namespace, map))
}

impl LoadedSoundBank {
    fn gap(&mut self, namespace: AssetNamespace, zone: &str, reason: String) {
        self.gaps.push(SoundZoneGap {
            namespace,
            zone: zone.to_owned(),
            reason,
        });
    }

    pub fn gap_lines(&self) -> Vec<String> {
        self.gaps
            .iter()
            .map(|g| {
                format!(
                    "sound bank gap: {} `{}` — {}",
                    g.namespace.as_str(),
                    g.zone,
                    g.reason
                )
            })
            .collect()
    }
}

/// The other games' main-menu music for the Black Ops II front end, as
/// sound catalogs to take in: BO1's (`common_mp`, from the games folder),
/// MW2's and MW3's (`localized_code_post_gfx_mp`, from the voice-only
/// folder, so neither game is otherwise loaded). One log line per source.
pub fn menu_music_guests(games: &GamesRoot) -> (Vec<SoundCatalog>, Vec<String>) {
    let voice = voice_games_root();
    let mut catalogs = Vec::new();
    let mut lines = Vec::new();
    let mut take = |label: &str, found: Result<std::path::PathBuf, String>, load: fn(&Path) -> Result<SoundCatalog, String>| {
        match found.and_then(|path| load(&path)) {
            Ok(catalog) => {
                lines.push(format!(
                    "menu music: {label} — {} aliases",
                    catalog.sounds.len()
                ));
                catalogs.push(catalog);
            }
            Err(error) => lines.push(format!("menu music: {label} not available ({error})")),
        }
    };
    // Black Ops' menu sounds (`uin_navigation_*`) are in whichever of
    // these carries its interface bank; each is taken when it loads.
    for zone in ["common_mp", "code_post_gfx_mp", "ui_mp"] {
        take(
            &format!("Black Ops {zone}"),
            find_zone_file_version(games, zone, fastfile_t5::ZONE_VERSION_PC).map(|z| z.path),
            crate::sound_load_t5::load_sound_catalog_t5,
        );
    }
    for (label, version, load) in [
        (
            "Modern Warfare 2 localized_code_post_gfx_mp",
            fastfile_iw4::ZONE_VERSION_PC,
            load_sound_catalog as fn(&Path) -> Result<SoundCatalog, String>,
        ),
        (
            "Modern Warfare 3 localized_code_post_gfx_mp",
            fastfile_iw5::ZONE_VERSION_PC,
            crate::sound_load_iw5::load_sound_catalog_iw5,
        ),
    ] {
        let found = voice
            .as_ref()
            .ok_or_else(|| "IW4L_VOICE_GAMES is not set".to_owned())
            .and_then(|root| find_zone_file_version(root, "localized_code_post_gfx_mp", version))
            .map(|z| z.path);
        take(label, found, load);
    }
    // bo2mp: Modern Warfare 3's menu sounds (`mouse_over`, `mouse_click`,
    // `mp_ui_decline`) for the shell's menus, from its interface zones.
    for zone in ["code_post_gfx_mp", "ui_mp"] {
        let found = voice
            .as_ref()
            .ok_or_else(|| "IW4L_VOICE_GAMES is not set".to_owned())
            .and_then(|root| find_zone_file_version(root, zone, fastfile_iw5::ZONE_VERSION_PC))
            .map(|z| z.path);
        take(&format!("Modern Warfare 3 {zone} (menu sounds)"), found, crate::sound_load_iw5::load_sound_catalog_iw5);
    }
    (catalogs, lines)
}
