use super::*;

#[derive(Clone, Debug, Default)]
pub struct MatchMaterialSeed {
    pub catalog: MaterialCatalog,
}

pub fn load_match_material_seed(
    progress: &LoadProgress,
) -> Result<(MatchMaterialSeed, Vec<String>), String> {
    let root = games_root_from_env()?;
    let runtime = find_zone_file_version(&root, "common_mp", fastfile_iw4::ZONE_VERSION_PC)?;

    let (catalog, mut report, _, _, _) = bevy::tasks::futures_lite::future::block_on(
        walk_startup_material_zones(Some(&runtime.path), progress),
    );
    let catalog =
        walk_t5_leftover_materials(Some(runtime.path.as_path()), progress, catalog, &mut report);
    let (catalog, common_report) = walk_material_file(&runtime.path, progress, catalog);
    report.extend(common_report);
    report.push(format!(
        "match material seed: materials={} techsets={} shaders={} decls={} decoded={}",
        catalog.materials.len(),
        catalog.technique_set_facts().len(),
        catalog.shaders.len(),
        catalog.vertex_decls.len(),
        catalog.image_memory().decoded_images,
    ));
    Ok((MatchMaterialSeed { catalog }, report))
}

pub fn apply_match_material_map(
    seed: &MatchMaterialSeed,
    zone_ff: &Path,
    progress: &LoadProgress,
) -> (MaterialCatalog, Vec<String>) {
    let mut report = Vec::new();
    let runtime = games_root_from_env()
        .ok()
        .and_then(|root| find_runtime_common_mp(&root, zone_ff).ok());
    let (foreign, foreign_report) = walk_foreign_material_population(
        Some(zone_ff),
        runtime.as_ref().map(|found| found.path.as_path()),
        progress,
    );
    report.extend(foreign_report);
    let mut catalog = seed.catalog.clone();
    catalog.absorb_asset_population(foreign);

    let common_techsets = catalog.technique_set_facts().to_vec();
    let (mut catalog, map_report) = walk_material_file(zone_ff, progress, catalog);
    report.extend(map_report);
    let absorbed = catalog.absorb_technique_set_tables(&common_techsets);
    let promoted = catalog.promote_iw5_fallback_tables();
    let t5_alias = catalog.absorb_t5_feature_token_donors();
    let stub_routed = catalog.reroute_stub_materials();
    report.push(format!(
        "material route (census): absorbed_techsets={absorbed} t5_tech_alias={t5_alias} iw5_promoted={promoted} stub_routed={stub_routed} unrouted={}",
        catalog.unrouted_material_count()
    ));
    (catalog, report)
}

pub fn load_match_material_catalog(
    zone_ff: &Path,
    progress: &LoadProgress,
) -> Result<(MaterialCatalog, Vec<String>), String> {
    let (seed, mut report) = load_match_material_seed(progress)?;
    let (catalog, map_report) = apply_match_material_map(&seed, zone_ff, progress);
    report.extend(map_report);
    Ok((catalog, report))
}

fn walk_material_file(
    path: &Path,
    progress: &LoadProgress,
    seed: MaterialCatalog,
) -> (MaterialCatalog, Vec<String>) {
    let population = walk_population_file(path, progress, seed);
    (population.materials, population.report)
}

pub(super) fn walk_population_file(
    path: &Path,
    progress: &LoadProgress,
    seed: MaterialCatalog,
) -> crate::lane::MaterialPopulation {
    let zone_name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "zone".into());
    let stage = progress.begin_scoped(StageId::CommonAssets, zone_name.clone(), None);
    let opened = open_zone_shared(path);
    finish_zone_open(stage, &opened);
    match opened {
        Ok(image) => lane(image.game).load_material_population(path, &image, progress, seed),
        Err(error) => crate::lane::MaterialPopulation {
            materials: seed,
            report: vec![format!("match materials: open {}: {error}", path.display())],
            ..Default::default()
        },
    }
}

fn walk_t5_leftover_materials(
    runtime_common: Option<&Path>,
    progress: &LoadProgress,
    seed: MaterialCatalog,
    report: &mut Vec<String>,
) -> MaterialCatalog {
    let root = match games_root_from_env() {
        Ok(root) => root,
        Err(error) => {
            report.push(format!("t5 leftover common: {error}"));
            return seed;
        }
    };
    let donor = match find_common_mp_for_envelope(&root, fastfile_t5::ZONE_VERSION_PC) {
        Ok(donor) => donor,
        Err(error) => {
            report.push(format!("t5 leftover common: {error}"));
            return seed;
        }
    };
    if runtime_common.is_some_and(|path| path == donor.path.as_path()) {
        report.push(format!(
            "t5 leftover common: runtime path is already {} — skip",
            donor.path.display()
        ));
        return seed;
    }
    let (leftover_startup, _, leftover_startup_report) =
        leftover_walk_t5_startup_materials(progress);
    report.extend(leftover_startup_report);
    let leftover_startup_n = leftover_startup.materials.len();
    let mut seed = seed;
    seed.absorb_missing_reals(leftover_startup);
    report.push(format!(
        "t5 leftover startup gfx absorb_missing_reals: donor={leftover_startup_n} host now {}",
        seed.materials.len()
    ));
    let (catalog, walked) = walk_material_file(&donor.path, progress, seed);
    report.extend(walked);
    report.push(format!(
        "t5 leftover common: path={} materials={}",
        donor.path.display(),
        catalog.materials.len(),
    ));
    catalog
}

fn leftover_walk_t5_startup_materials(
    progress: &LoadProgress,
) -> (
    MaterialCatalog,
    Vec<asset_game::CapturedStringTable>,
    Vec<String>,
) {
    let mut report = Vec::new();
    let root = match games_root_from_env() {
        Ok(root) => root,
        Err(error) => {
            report.push(format!("t5 leftover startup gfx: {error}"));
            return (MaterialCatalog::default(), Vec::new(), report);
        }
    };
    const ZONES: [&str; 3] = ["code_post_gfx_mp", "localized_code_post_gfx_mp", "patch_mp"];
    let mut seed = MaterialCatalog::default();
    let mut stats = Vec::new();
    for zone in ZONES {
        let found = match find_zone_file_version(&root, zone, fastfile_t5::ZONE_VERSION_PC) {
            Ok(found) => found,
            Err(error) => {
                report.push(format!("t5 leftover startup gfx gap: {zone}: {error}"));
                continue;
            }
        };
        let population = walk_population_file(&found.path, progress, seed);
        report.extend(population.report);
        if zone == "code_post_gfx_mp" {
            stats = population.cac_tables;
        }
        seed = population.materials;
    }
    report.push(format!(
        "t5 leftover startup gfx: materials={}",
        seed.materials.len()
    ));
    (seed, stats, report)
}

/// bo2mp (MW3 maps glitched: floating cards, opaque decals, black floor
/// holes): the `common_mp` of the map zone's own game. `find_common_mp_for_zone`
/// searches every configured game folder and prefers an IW4 (MW2)
/// `common_mp`, so for an MW3 map it named MW2's, the runtime common, and
/// the donor was dropped as "already walked": MW3's `common_mp`, which holds
/// the technique sets and shared images MW3's map materials point at, was
/// never walked (`TechniqueSetMissing` for ~1150 materials on Off Shore).
/// An IW5 map's donor is the IW5 `common_mp`, searched from its own tree.
pub(super) fn same_game_common_mp(zone_ff: &Path) -> Result<asset_transport::ZoneFile, String> {
    if peek_zone_version(zone_ff) == Some(fastfile_iw5::ZONE_VERSION_PC) {
        let tree = asset_transport::game_root_for_zone(zone_ff)?;
        return find_zone_file_version(
            &asset_transport::GamesRoot(tree),
            "common_mp",
            fastfile_iw5::ZONE_VERSION_PC,
        );
    }
    find_common_mp_for_zone(zone_ff)
}

fn walk_foreign_material_population(
    zone_ff: Option<&Path>,
    runtime_common: Option<&Path>,
    progress: &LoadProgress,
) -> (MaterialCatalog, Vec<String>) {
    let mut report = Vec::new();
    let Some(zone_ff) = zone_ff else {
        return (MaterialCatalog::default(), report);
    };
    let donor = match same_game_common_mp(zone_ff) {
        Ok(donor) => donor,
        Err(error) => {
            report.push(format!(
                "foreign data common: no same-tree common_mp ({error})"
            ));
            return (MaterialCatalog::default(), report);
        }
    };
    if runtime_common.is_some_and(|path| path == donor.path.as_path()) {
        return (MaterialCatalog::default(), report);
    }
    match peek_zone_version(&donor.path) {
        Some(fastfile_iw5::ZONE_VERSION_PC) => {}
        Some(version) => {
            report.push(format!(
                "foreign data common: skip envelope {version:#x} at {} (IW5 material data only)",
                donor.path.display()
            ));
            return (MaterialCatalog::default(), report);
        }
        None => {
            report.push(format!(
                "foreign data common: unreadable envelope at {}",
                donor.path.display()
            ));
            return (MaterialCatalog::default(), report);
        }
    }
    let (materials, walked) = walk_material_file(&donor.path, progress, MaterialCatalog::default());
    report.extend(walked);
    report.push(format!(
        "foreign data common: path={} materials={} techsets={} (weapons discarded)",
        donor.path.display(),
        materials.materials.len(),
        materials.technique_set_facts().len(),
    ));
    (materials, report)
}

pub(super) fn resolve_foreign_material_donor(
    zone_ff: Option<&Path>,
    runtime_common: Option<&Path>,
    report: &mut Vec<String>,
) -> Option<PathBuf> {
    let zone_ff = zone_ff?;
    let donor = match same_game_common_mp(zone_ff) {
        Ok(donor) => donor,
        Err(error) => {
            report.push(format!(
                "foreign data common: no same-tree common_mp ({error})"
            ));
            return None;
        }
    };
    if runtime_common.is_some_and(|path| path == donor.path.as_path()) {
        return None;
    }
    match peek_zone_version(&donor.path) {
        Some(fastfile_iw5::ZONE_VERSION_PC) => {
            report.push(format!(
                "foreign data common: IW5 common_mp {} for {}",
                donor.path.display(),
                zone_ff.display()
            ));
            Some(donor.path)
        }
        Some(version) => {
            report.push(format!(
                "foreign data common: skip envelope {version:#x} at {} (IW5 material data only)",
                donor.path.display()
            ));
            None
        }
        None => {
            report.push(format!(
                "foreign data common: unreadable envelope at {}",
                donor.path.display()
            ));
            None
        }
    }
}

pub(super) fn walk_foreign_material_common(
    donor: &Path,
    progress: &LoadProgress,
    job: load_jobs::Job,
) -> (MaterialCatalog, Option<PendingImages>, Vec<String>) {
    let mut report = Vec::new();
    let Some(mut census) = capture_common_zone(
        donor,
        "walking same-tree common_mp as material data",
        "foreign data common",
        progress,
        &mut report,
    ) else {
        return (MaterialCatalog::default(), None, report);
    };
    let pending_images = hold_image_plan("IW5 common_mp", census.pending_images.take(), job)
        .map(|held| held.enqueue(progress));
    let materials = census.material_population;
    report.extend(census.report);
    report.push(format!(
        "foreign data common: path={} materials={} techsets={} (weapons discarded)",
        donor.display(),
        materials.materials.len(),
        materials.technique_set_facts().len(),
    ));
    (materials, pending_images, report)
}

fn capture_common_zone(
    donor: &Path,
    stage_label: &str,
    report_label: &str,
    progress: &LoadProgress,
    report: &mut Vec<String>,
) -> Option<crate::lane::CommonCensus> {
    if progress.is_canceled() {
        report.push(format!("{report_label}: canceled before the walk started"));
        return None;
    }
    let stage = progress.begin_scoped(StageId::CommonAssets, stage_label, None);
    let opened = open_zone_shared(donor);
    finish_zone_open(stage, &opened);
    match opened {
        Ok(image) => Some(lane(image.game).load_common_mp(
            donor,
            &image,
            progress,
            true,
            MaterialCatalog::default(),
        )),
        Err(error) => {
            report.push(format!("{report_label}: open {}: {error}", donor.display()));
            None
        }
    }
}

pub(super) fn merge_image_batch(
    global: &mut asset_material::MaterialDefinitions,
    label: &str,
    batch: asset_material::material_images::DecodedImageBatch,
    job: load_jobs::Job,
    report: &mut Vec<String>,
) {
    let requested = batch.stats.requested;
    let missing = batch.stats.missing;
    let unsupported = batch.stats.unsupported;
    let first_gap = batch.stats.first_gap.clone();
    let census = batch.apply(global);
    job.bytes(None, Some(census.final_cpu_bytes))
        // What the plan prepared and what it served out of another plan's work,
        // kept apart: only the first is work this plan did.
        .prepared(census.newly_prepared_bytes, census.reused_bytes)
        .merged(
            census.final_cpu_bytes,
            census.discarded_decoded_bytes,
            census.discard_line(),
        );
    report.push(format!(
            "{label} claimed images: {}/{requested} into the merged pool ({} already decoded by an earlier source, {missing} missing, {unsupported} unsupported, {} claimed rows dropped by the merge)",
            census.filled_rows, census.already_decoded, census.discarded_variants,
        ));
    report.push(format!(
        "{label} image demand: claimed_rows={} canonical_variants={} prepared_variants={} duplicate_claims={} pruned_variants={} pruned_rows={} final_cpu_bytes={} newly_prepared_bytes={} reused_variants={} reused_bytes={} discarded_decoded_bytes={} discarded_same_payload={}",
        census.claimed_rows,
        census.canonical_variants,
        census.prepared_variants,
        census.duplicate_claims,
        census.pruned_variants,
        census.pruned_rows,
        census.final_cpu_bytes,
        census.newly_prepared_bytes,
        census.reused_variants,
        census.reused_bytes,
        census.discarded_decoded_bytes,
        census.discarded_same_payload,
    ));
    if let Some(line) = census.discard_line() {
        report.push(format!("{label} image discard: {line}"));
    }
    // Who answered for the claims this plan prepared and the merge threw away,
    // which is what says whether a claim could have been resolved earlier.
    if !census.disputed_winners.is_empty() {
        report.push(format!(
            "{label} image dispute: {}",
            census
                .disputed_winners
                .iter()
                .map(|winner| format!(
                    "{}={} won_by={} first={}",
                    winner.kind,
                    winner.claims,
                    winner
                        .plan
                        .map_or_else(|| "none".to_owned(), |plan| format!("plan{plan}")),
                    winner.first,
                ))
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    if let Some(gap) = first_gap {
        report.push(format!("{label} claimed image gap: {gap}"));
    }
}

/// An image plan already on the pool, and the job row that records when it got
/// there. The walk that discovers the demand hands both over itself.
pub(super) struct PendingImages {
    pub(super) job: load_jobs::Job,
    pub(super) task: bevy::tasks::Task<(
        &'static str,
        asset_material::material_images::DecodedImageBatch,
    )>,
}

impl PendingImages {
    pub(super) async fn join(
        self,
    ) -> (
        &'static str,
        asset_material::material_images::DecodedImageBatch,
    ) {
        let done = self.task.await;
        self.job.joined();
        done
    }
}

/// A plan whose walk has finished, holding the job row that records when it did.
///
/// `job` is opened by the producer when it starts looking, so the row carries
/// discovered → ready → enqueued → started → finished → joined. A plan waiting
/// for the catalog spends that wait in `ready → enqueued`, not in a decode
/// timer.
pub(super) struct HeldImagePlan {
    pub(super) label: &'static str,
    pub(super) plan: ImageDemandPlan,
    pub(super) job: load_jobs::Job,
}

/// Take a finished walk's plan, if it wants anything at all.
pub(super) fn hold_image_plan(
    label: &'static str,
    plan: Option<ImageDemandPlan>,
    job: load_jobs::Job,
) -> Option<HeldImagePlan> {
    let plan = plan.filter(|plan| !plan.is_empty())?;
    // The census names the winner of a disputed claim by plan id, so the id
    // and the label are printed together once, here, where both are known.
    diag::info!(
        Zone,
        "image plan: plan{} is {label} ({} canonical variants, {} claimed rows)",
        plan.id(),
        plan.canonical_variants(),
        plan.claimed_rows(),
    );
    let job = job
        .canonical(label)
        .items(plan.canonical_variants() as u64)
        .plan_ready();
    Some(HeldImagePlan { label, plan, job })
}

impl HeldImagePlan {
    /// Put the plan on the load pool now, with every claim it made.
    fn enqueue(self, progress: &LoadProgress) -> PendingImages {
        let Self { label, plan, job } = self;
        let job = job.enqueued();
        let progress = progress.clone();
        let task = load_pool().spawn(async move {
            job.started();
            let stage = progress.begin_scoped(StageId::Images, label, None);
            // The plan splits itself across this same pool. The worker this
            // task is on joins that scope rather than blocking on it, so a
            // plan does not cost a load worker to supervise it.
            let batch = plan.run(&stage, job, load_pool());
            stage.done();
            job.finished();
            (label, batch)
        });
        PendingImages { job, task }
    }

    /// Resolve the plan's claims against the merged catalog, then decode what
    /// is left of it.
    ///
    /// The catalog has to be one every rival plan has already applied into: a
    /// row still carrying this plan's id with nothing decoded in it is a row
    /// this plan will fill, and a name with no such row left is a decode whose
    /// result the merge would count and throw away. Asked earlier, the same
    /// question cuts names a plan still in flight is about to release.
    ///
    /// Returns `None` when nothing survives, which is a plan that never runs
    /// rather than an empty one that does.
    pub(super) fn prune_then_enqueue(
        mut self,
        catalog: &mut asset_material::MaterialDefinitions,
        progress: &LoadProgress,
        report: &mut Vec<String>,
    ) -> Option<PendingImages> {
        let resolving = std::time::Instant::now();
        self.plan.prune_to(catalog);
        let resolved_ms = resolving.elapsed().as_secs_f32() * 1000.0;
        let (variants, rows) = self.plan.pruned();
        // The cost of asking is on the line beside what it saved: this runs
        // on the consumer, between the last donor's apply and the decode, so
        // it is serial time the walk pays whatever the decode then skips.
        report.push(format!(
            "{} image claims resolved before decode: {} of {} variants cut ({rows} claimed rows), {} left to decode, {resolved_ms:.1}ms to resolve",
            self.label,
            variants,
            self.plan.canonical_variants(),
            self.plan.len(),
        ));
        if self.plan.is_empty() {
            // Nothing left to decode means nothing will ever call `apply`, and
            // `apply` is what hands the claimed rows back. Do it here instead,
            // or the rows keep a mark saying a decode is owed on them and the
            // stages after this one skip every name the plan had claimed.
            let released = self.plan.release_claims(catalog);
            report.push(format!(
                "{} image plan dropped before decode: every claim was answered elsewhere, {released} claimed rows handed back",
                self.label,
            ));
            self.job.items(0).enqueued().started().finished().joined();
            return None;
        }
        Some(self.enqueue(progress))
    }
}

pub(super) enum ForeignCommonWork {
    Shared(
        bevy::tasks::Task<(
            MaterialCatalog,
            Iw5WeaponBundle,
            Option<PendingImages>,
            Vec<String>,
        )>,
    ),
    Split(Option<bevy::tasks::Task<(MaterialCatalog, Option<PendingImages>, Vec<String>)>>),
}

#[derive(Default)]
pub(super) struct Iw5WeaponBundle {
    pub(super) weapons: WeaponBuild,
    pub(super) fpv: FpvMeshBuild,
    pub(super) world_guns: WorldWeaponBuild,
    pub(super) xanims: XAnimBuild,

    pub(super) materials: MaterialCatalog,
    pub(super) stats_tables: Vec<asset_game::CapturedStringTable>,
    pub(super) scene_models: asset_world::MapXModelSceneCatalog,
    pub(super) shared_surfaces: asset_model::SharedXModelSurfaces,
}

pub(super) fn resolve_iw5_weapon_donor(
    runtime_common: Option<&Path>,
    report: &mut Vec<String>,
) -> Option<PathBuf> {
    let Ok(root) = games_root_from_env() else {
        report.push("iw5 weapons: IW4L_GAMES unset".into());
        return None;
    };
    let donor = match find_zone_file_version(&root, "common_mp", fastfile_iw5::ZONE_VERSION_PC) {
        Ok(donor) => donor,
        Err(error) => {
            report.push(format!("iw5 weapons: {error}"));
            return None;
        }
    };
    if runtime_common.is_some_and(|path| path == donor.path.as_path()) {
        report.push(format!(
            "iw5 weapons: skip, runtime common is already IW5 ({})",
            donor.path.display()
        ));
        return None;
    }
    Some(donor.path)
}

pub(super) fn walk_iw5_weapon_bundle(
    donor: &Path,
    progress: &LoadProgress,
    job: load_jobs::Job,
) -> (Iw5WeaponBundle, Option<PendingImages>, Vec<String>) {
    let mut report = Vec::new();
    let Some(mut census) = capture_common_zone(
        donor,
        "walking IW5 common_mp as weapon bundle",
        "iw5 weapons",
        progress,
        &mut report,
    ) else {
        return (Iw5WeaponBundle::default(), None, report);
    };
    // The producer enqueues the bundle's images itself. Left for the consumer,
    // they wait out the synchronous `common_mp` walk and the unpacking of this
    // tuple: seconds of ready decode work with nobody holding it.
    let pending_images = hold_image_plan("IW5 weapon bundle", census.pending_images.take(), job)
        .map(|held| held.enqueue(progress));
    report.extend(census.report);
    report.push(format!(
        "iw5 weapons: path={} ids={} gun_named={} fpv={} world_guns={} xanims={} materials={}",
        donor.display(),
        census.weapons.len(),
        census.weapons.gun_xmodel_count(),
        census.fpv.len(),
        census.world_weapons.len(),
        census.xanims.len(),
        census.material_population.materials.len(),
    ));
    (
        Iw5WeaponBundle {
            weapons: census.weapons,
            fpv: census.fpv,
            world_guns: census.world_weapons,
            xanims: census.xanims,
            materials: census.material_population,
            stats_tables: census.cac_tables,
            scene_models: census.scene_models,
            shared_surfaces: census.shared_surfaces,
        },
        pending_images,
        report,
    )
}

pub(super) fn walk_shared_iw5_common(
    donor: &Path,
    progress: &LoadProgress,
    job: load_jobs::Job,
) -> (
    MaterialCatalog,
    Iw5WeaponBundle,
    Option<PendingImages>,
    Vec<String>,
) {
    let mut report = Vec::new();
    let Some(mut census) = capture_common_zone(
        donor,
        "walking IW5 common_mp as material data + weapon bundle",
        "iw5 common",
        progress,
        &mut report,
    ) else {
        return (
            MaterialCatalog::default(),
            Iw5WeaponBundle::default(),
            None,
            report,
        );
    };
    // One capture serves the material seed and the weapon bundle, so there is
    // one plan here, not two: the donor is walked once and its images are
    // claimed once.
    let pending_images = hold_image_plan("IW5 common_mp", census.pending_images.take(), job)
        .map(|held| held.enqueue(progress));
    report.extend(census.report);
    let materials = census.material_population;
    report.push(format!(
        "iw5 common (shared capture): path={} materials={} techsets={} ids={} gun_named={} fpv={} world_guns={} xanims={} — one read/inflate/walk serves both the material seed and the weapon bundle; leftover materials are the seeded rows, not a second donor",
        donor.display(),
        materials.materials.len(),
        materials.technique_set_facts().len(),
        census.weapons.len(),
        census.weapons.gun_xmodel_count(),
        census.fpv.len(),
        census.world_weapons.len(),
        census.xanims.len(),
    ));

    (
        materials,
        Iw5WeaponBundle {
            weapons: census.weapons,
            fpv: census.fpv,
            world_guns: census.world_weapons,
            xanims: census.xanims,
            materials: MaterialCatalog::default(),
            stats_tables: census.cac_tables,
            scene_models: census.scene_models,
            shared_surfaces: census.shared_surfaces,
        },
        pending_images,
        report,
    )
}

pub(super) enum T5CommonPrep {
    Skip(Vec<String>),
    Ready {
        donor: PathBuf,
        opened: Result<std::sync::Arc<asset_transport::ZoneImage>, String>,
        leftover_startup: MaterialCatalog,
        leftover_stats: Vec<asset_game::CapturedStringTable>,
        report: Vec<String>,
    },
}

pub(super) fn t5_weapon_common_prep(
    runtime_common: Option<&Path>,
    progress: &LoadProgress,
) -> T5CommonPrep {
    let mut report = Vec::new();
    let root = match games_root_from_env() {
        Ok(root) => root,
        Err(error) => {
            report.push(format!("t5 weapon common: {error}"));
            return T5CommonPrep::Skip(report);
        }
    };
    let donor = match find_common_mp_for_envelope(&root, fastfile_t5::ZONE_VERSION_PC) {
        Ok(donor) => donor,
        Err(error) => {
            report.push(format!("t5 weapon common: {error}"));
            return T5CommonPrep::Skip(report);
        }
    };
    if runtime_common.is_some_and(|path| path == donor.path.as_path()) {
        report.push(format!(
            "t5 weapon common: runtime path is already {} — skip absorb",
            donor.path.display()
        ));
        return T5CommonPrep::Skip(report);
    }
    let (leftover_startup, leftover_stats, leftover_startup_report) =
        leftover_walk_t5_startup_materials(progress);
    report.extend(leftover_startup_report);
    let stage = progress.begin_scoped(StageId::CommonAssets, "t5_weapons", None);
    let opened = open_zone_shared(&donor.path).map_err(|error| error.to_string());
    stage.finish_from(&opened);
    T5CommonPrep::Ready {
        donor: donor.path,
        opened,
        leftover_startup,
        leftover_stats,
        report,
    }
}

/// bo2mp: Modern Warfare 2's guns as extra weapons in a Black Ops II
/// match whose map is not one that already loads MW2's common zone (a BO2,
/// CoD4 or WaW map): its common_mp walked for its guns as Black Ops' is.
pub(super) fn iw4_guest_common_prep(runtime_common: Option<&Path>, progress: &LoadProgress) -> T5CommonPrep {
    let mut report = Vec::new();
    let root = match games_root_from_env() {
        Ok(root) => root,
        Err(error) => {
            report.push(format!("iw4 guest guns: {error}"));
            return T5CommonPrep::Skip(report);
        }
    };
    let donor = match find_common_mp_for_envelope(&root, fastfile_iw4::ZONE_VERSION_PC) {
        Ok(donor) => donor,
        Err(error) => {
            report.push(format!("iw4 guest guns: {error}"));
            return T5CommonPrep::Skip(report);
        }
    };
    if runtime_common.is_some_and(|path| path == donor.path.as_path()) {
        report.push(format!("iw4 guest guns: runtime common is already MW2's ({}), its guns are in", donor.path.display()));
        return T5CommonPrep::Skip(report);
    }
    let stage = progress.begin_scoped(StageId::CommonAssets, "iw4_guest_weapons", None);
    let opened = open_zone_shared(&donor.path).map_err(|error| error.to_string());
    stage.finish_from(&opened);
    report.push(format!("iw4 guest guns: walking {}", donor.path.display()));
    T5CommonPrep::Ready {
        donor: donor.path,
        opened,
        leftover_startup: MaterialCatalog::default(),
        leftover_stats: Vec::new(),
        report,
    }
}

pub(super) struct T5WeaponCommon {
    pub(super) weapons: WeaponBuild,
    pub(super) fpv: FpvMeshBuild,
    pub(super) world_guns: WorldWeaponBuild,
    pub(super) material_seed: MaterialCatalog,
    pub(super) xanims: XAnimBuild,
    pub(super) fx: FxCatalog,
    pub(super) impact_fx: Option<asset_game::OwnedFxImpactTable>,
    pub(super) projectiles: asset_model::ProjectileMeshBuild,
    pub(super) teamsets: std::collections::HashMap<String, asset_game::MapTeamSettings>,
    pub(super) scene_models: asset_world::MapXModelSceneCatalog,
    pub(super) images: Option<PendingImages>,
    pub(super) stats_tables: (
        Vec<asset_game::CapturedStringTable>,
        Vec<asset_game::CapturedStringTable>,
    ),
    pub(super) report: Vec<String>,
}

impl T5WeaponCommon {
    pub(super) fn empty(material_seed: MaterialCatalog, report: Vec<String>) -> Self {
        Self {
            weapons: asset_game::WeaponBuild::default(),
            fpv: FpvMeshBuild::default(),
            world_guns: WorldWeaponBuild::default(),
            material_seed,
            xanims: XAnimBuild::default(),
            fx: FxCatalog::default(),
            impact_fx: None,
            projectiles: asset_model::ProjectileMeshBuild::default(),
            teamsets: Default::default(),
            scene_models: Default::default(),
            images: None,
            stats_tables: (Vec::new(), Vec::new()),
            report,
        }
    }
}

pub(super) fn walk_t5_weapon_common(
    prep: T5CommonPrep,
    progress: &LoadProgress,
    material_seed: MaterialCatalog,
    job: load_jobs::Job,
) -> T5WeaponCommon {
    let (donor, opened, leftover_startup, leftover_stats, mut report) = match prep {
        T5CommonPrep::Skip(report) => return T5WeaponCommon::empty(material_seed, report),
        T5CommonPrep::Ready {
            donor,
            opened,
            leftover_startup,
            leftover_stats,
            report,
        } => (donor, opened, leftover_startup, leftover_stats, report),
    };
    let leftover_startup_n = leftover_startup.materials.len();
    let mut material_seed = material_seed;
    material_seed.absorb_missing_reals(leftover_startup);
    report.push(format!(
        "t5 leftover startup gfx absorb_missing_reals: donor={leftover_startup_n} host now {}",
        material_seed.materials.len()
    ));
    let image = match opened {
        Ok(image) => image,
        Err(error) => {
            report.push(format!(
                "t5 weapon common: open {}: {error}",
                donor.display()
            ));
            let mut empty = T5WeaponCommon::empty(material_seed, report);
            empty.stats_tables.0 = leftover_stats;
            return empty;
        }
    };
    let mut census = lane(image.game).load_common_mp(&donor, &image, progress, true, material_seed);
    let images = hold_image_plan("T5 common_mp", census.pending_images.take(), job)
        .map(|held| held.enqueue(progress));
    report.extend(census.report);
    report.push(format!(
        "t5 weapon common: path={} weapons={} fpv={} world_guns={} materials={} xanims={} fx={}",
        donor.display(),
        census.weapons.len(),
        census.fpv.len(),
        census.world_weapons.len(),
        census.material_population.materials.len(),
        census.xanims.len(),
        census.fx.len(),
    ));
    T5WeaponCommon {
        weapons: census.weapons,
        fpv: census.fpv,
        world_guns: census.world_weapons,
        material_seed: census.material_population,
        xanims: census.xanims,
        fx: census.fx,
        impact_fx: census.impact_fx,
        projectiles: census.projectile_meshes,
        teamsets: census.teamsets,
        scene_models: census.scene_models,
        images,
        stats_tables: (leftover_stats, census.cac_tables),
        report,
    }
}

pub(super) async fn walk_startup_material_zones(
    map_path: Option<&PathBuf>,
    progress: &LoadProgress,
) -> (
    MaterialCatalog,
    Vec<String>,
    Vec<asset_game::CapturedStringTable>,
    Vec<asset_world::CapturedLightDef>,
    crate::ScriptSources,
) {
    const STARTUP_ZONES: [&str; 3] = ["code_post_gfx_mp", "localized_code_post_gfx_mp", "patch_mp"];
    let Some(map_path) = map_path else {
        return (
            MaterialCatalog::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            crate::ScriptSources::default(),
        );
    };
    let games = games_root_from_env().ok();

    let opened = STARTUP_ZONES
        .map(|zone| {
            let games = games.clone();
            let map_path = map_path.clone();
            let progress = progress.clone();
            load_pool().spawn(async move {
                let found = match &games {
                    Some(root) => find_runtime_zone(root, &map_path, zone),
                    None => find_zone_for_tree(&map_path, zone),
                };
                found.and_then(|found| {
                    let stage = progress.begin_scoped(StageId::CommonAssets, zone, None);
                    let image = open_zone_shared(&found.path)
                        .map(|image| (found.path, image))
                        .map_err(|error| error.to_string());
                    if let Ok((_, image)) = &image {
                        stage.set_bytes(image.bytes.len() as u64);
                    }
                    stage.finish_from(&image);
                    image
                })
            })
        })
        .into_iter();
    let mut seed = MaterialCatalog::default();
    let mut report = Vec::new();
    let mut reuse_mat = 0usize;
    let mut reuse_img = 0usize;
    let mut zones_ok = 0usize;
    let mut stats = Vec::new();
    let mut light_defs = Vec::new();
    let mut scripts = crate::ScriptSources::default();
    for (zone, task) in STARTUP_ZONES.into_iter().zip(opened) {
        match task.await {
            Ok((path, image)) => {
                let envelope = peek_zone_version(&path)
                    .map(|v| format!("{v:#x}"))
                    .unwrap_or_else(|| "unreadable".into());
                let pop = lane(image.game).load_material_population(&path, &image, progress, seed);
                report.extend(pop.report);
                reuse_mat = reuse_mat.saturating_add(pop.materials.link_reused_materials);
                reuse_img = reuse_img.saturating_add(pop.materials.link_reused_images);
                report.push(format!(
                    "material generation startup: {zone} {} materials images={} decoded={} envelope={envelope} path={}",
                    pop.materials.materials.len(),
                    pop.materials.images.len(),
                    pop.materials.image_memory().decoded_images,
                    path.display()
                ));
                seed = pop.materials;
                light_defs.extend(pop.light_defs);
                scripts.overlay(pop.scripts);
                if zone == "code_post_gfx_mp" {
                    stats = pop.cac_tables;
                }
                zones_ok += 1;
            }
            Err(gap) => report.push(format!("material generation startup gap: {zone}: {gap}")),
        }
    }
    let startup_decoded = seed.image_memory().decoded_images;
    report.push(format!(
        "startup material decode: zones={zones_ok} decoded_images={startup_decoded} (0 is IWD deferred to absorb merge; not three parallel common_mp FPV decodes)",
    ));
    report.push(format!(
        "startup material walk: zones={zones_ok} fpv=0 weapons=0 (materials-only sink; not load_common_mp)",
    ));
    report.push(format!(
        "s2 startup walk: reuse_mat={reuse_mat} reuse_img={reuse_img} pool_mat={} pool_img={}",
        seed.materials.len(),
        seed.images.len(),
    ));
    (seed, report, stats, light_defs, scripts)
}

pub(super) fn load_localized_strings_beside(
    zone_ff: &std::path::Path,
    report: &mut Vec<String>,
    stage: &asset_transport::progress::StageHandle,
) -> LocalizeCatalog {
    let mut catalog = LocalizeCatalog::default();
    let root = match games_root_from_env() {
        Ok(root) => root,
        Err(error) => {
            report.push(format!("localize gap: IW4L_GAMES unset ({error})"));
            return catalog;
        }
    };
    let lanes: [(asset_core::AssetNamespace, u32, &[&str]); 3] = [
        (
            asset_core::AssetNamespace::Iw4,
            fastfile_iw4::ZONE_VERSION_PC,
            MP_LOCALIZED_ZONES,
        ),
        (
            asset_core::AssetNamespace::T5,
            fastfile_t5::ZONE_VERSION_PC,
            &["code_post_gfx_mp", "common_mp", "ui_mp"],
        ),
        (
            asset_core::AssetNamespace::Iw5,
            fastfile_iw5::ZONE_VERSION_PC,
            MP_LOCALIZED_ZONES,
        ),
    ];
    let runtime_language = find_runtime_common_mp(&root, zone_ff)
        .ok()
        .and_then(|zone| zone.path.parent()?.file_name()?.to_str().map(str::to_owned));
    let mut plan = Vec::new();
    for (namespace, version, names) in lanes {
        let found: Vec<_> = if namespace == asset_core::AssetNamespace::T5 {
            match find_zone_file_version(&root, "common_mp", version).and_then(|zone| {
                asset_transport::discover::find_t5_localized_zones(
                    &zone.path,
                    runtime_language.as_deref(),
                )
            }) {
                Ok(zones) => zones,
                Err(error) => {
                    report.push(format!("localize gap: {error}"));
                    Vec::new()
                }
            }
        } else {
            names
                .iter()
                .filter_map(|name| {
                    if namespace == asset_core::AssetNamespace::Iw4 {
                        find_runtime_zone(&root, zone_ff, name)
                    } else {
                        find_zone_file_version(&root, name, version)
                    }
                    .ok()
                })
                .collect()
        };
        plan.extend(found.into_iter().map(|found| (namespace, found)));
    }
    // The plan only closes once every lane has been searched: a zone that is
    // not there is not part of the volume, and counting it would report work
    // nobody is going to do.
    stage.set_total(plan.len() as u64);
    for (namespace, found) in plan {
        let name = &found.zone_name;
        match load_localize_catalog_in_lane(&found.path) {
            Ok(part) if part.is_empty() => {}
            Ok(part) => {
                report.push(format!(
                    "localize: {} {name} {} strings",
                    namespace.as_str(),
                    part.len()
                ));
                catalog.absorb_in_namespace(namespace, part);
            }
            Err(error) => report.push(format!(
                "localize gap: {} {name} failed: {error}",
                namespace.as_str()
            )),
        }
        // A zone that failed to open is still a unit this pass has handled.
        stage.advance(1);
    }
    report.push(format!(
        "localize: {} strings for on-screen text",
        catalog.len()
    ));
    catalog
}

/// Close a zone-open stage with the weight of what it read.
///
/// A zone open has no units to count — it is one read — but it does have a
/// size, and that size is what the rest of the load is built out of. Recording
/// it here means every opener says it the same way.
pub(super) fn finish_zone_open<E>(
    stage: StageHandle,
    opened: &Result<std::sync::Arc<asset_transport::zone::ZoneImage>, E>,
) {
    if let Ok(image) = opened {
        stage.set_bytes(image.bytes.len() as u64);
    }
    stage.finish_from(opened);
}

#[cfg(test)]
mod same_game_common_tests {
    use super::*;

    fn zone(path: &Path, version: u32) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut bytes = b"IWffu100".to_vec();
        bytes.extend_from_slice(&version.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 32]);
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn mw3_dlc_map_takes_mw3_common_mp_not_mw2s() {
        let root = std::env::temp_dir().join(format!(
            "bo2mp_same_game_common_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let mw3 = root.join("mw3_game_files");
        // An IW4 common_mp in the same tree is what the any-game search
        // prefers; the MW3 map must still get the IW5 one.
        zone(&mw3.join("zone/aaa_iw4/common_mp.ff"), fastfile_iw4::ZONE_VERSION_PC);
        zone(&mw3.join("zone/english/common_mp.ff"), fastfile_iw5::ZONE_VERSION_PC);
        let map = mw3.join("zone/dlc/mp_roughneck.ff");
        zone(&map, fastfile_iw5::ZONE_VERSION_PC);
        let found = same_game_common_mp(&map).expect("iw5 common_mp");
        assert_eq!(
            peek_zone_version(&found.path),
            Some(fastfile_iw5::ZONE_VERSION_PC),
            "{}",
            found.path.display()
        );
        assert!(found.path.ends_with("zone/english/common_mp.ff"));
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// bo2mp `iw4l check-guns`: reads Black Ops' guns the way a match does and
/// says, for each guest gun Create-a-Class lists, whether it loaded and
/// what it brings (variants, first-person and world models, animations,
/// missing pieces). One line each, then a summary.
pub fn check_guest_guns() -> Vec<String> {
    let mut out = Vec::new();
    for (game, ns) in [
        ("t5", asset_core::AssetNamespace::T5),
        ("iw4", asset_core::AssetNamespace::Iw4),
        ("iw5", asset_core::AssetNamespace::Iw5),
        ("t4", asset_core::AssetNamespace::T4),
        ("iw3", asset_core::AssetNamespace::Iw3),
        ("iw6", asset_core::AssetNamespace::Iw6),
    ] {
        // (`BO2MP_CHECK_GUNS=iw5,t4` checks only those games.)
        if std::env::var("BO2MP_CHECK_GUNS").is_ok_and(|only| !only.split(',').any(|g| g.trim() == game)) {
            continue;
        }
        out.extend(check_guest_game(game, ns));
    }
    out
}

/// A guest game's guns read for `check-guns`: the weapons, the names of the
/// first-person meshes and of the animations its zone brought, the report.
struct GuestCheckLoad {
    weapons: WeaponBuild,
    fpv_names: std::collections::HashSet<String>,
    xanims: XAnimBuild,
    report: Vec<String>,
}

fn load_guest_check(game: &str, ns: asset_core::AssetNamespace, progress: &LoadProgress) -> GuestCheckLoad {
    if game == "iw5" {
        return load_iw5_guest_check(progress);
    }
    // bo2mp: Ghosts' guns, read by `fastfile_iw6` as a match reads them.
    if game == "iw6" {
        let mut guests = crate::lane::walk_iw6_guest_guns(true, MaterialCatalog::default(), progress);
        guests.report.extend(crate::lane::iw6_sound_check(&guests.weapons));
        guests.report.push(crate::lane::iw6_anim_check(&guests.xanims));
        let fpv_names = (0..guests.fpv.len()).filter_map(|i| guests.fpv.name_at(i).map(str::to_owned)).collect();
        return GuestCheckLoad { weapons: guests.weapons, fpv_names, xanims: guests.xanims, report: guests.report };
    }
    // bo2mp: World at War's and CoD4's guns, as a match reads them.
    if let Some(legacy) = [crate::lane::Legacy::T4, crate::lane::Legacy::Iw3].into_iter().find(|l| l.tag() == game) {
        let guests = match crate::lane::legacy_common_mp(legacy) {
            Ok(path) => crate::lane::load_legacy_guests(legacy, &path, MaterialCatalog::default()),
            Err(error) => crate::lane::LegacyGuests::empty(MaterialCatalog::default(), vec![format!("{game} guest guns: {error}")]),
        };
        let fpv_names = (0..guests.fpv.len()).filter_map(|i| guests.fpv.name_at(i).map(str::to_owned)).collect();
        return GuestCheckLoad { weapons: guests.weapons, fpv_names, xanims: guests.xanims, report: guests.report };
    }
    let prep = if game == "t5" { t5_weapon_common_prep(None, progress) } else { iw4_guest_common_prep(None, progress) };
    let job = load_jobs::open(JobKind::ImageDecode).namespace(ns.as_str());
    let common = walk_t5_weapon_common(prep, progress, MaterialCatalog::default(), job);
    let fpv_names = (0..common.fpv.len()).filter_map(|i| common.fpv.name_at(i).map(str::to_owned)).collect();
    GuestCheckLoad { weapons: common.weapons, fpv_names, xanims: common.xanims, report: common.report }
}

/// bo2mp: MW3's guns as a Black Ops II match reads them: its common_mp's
/// weapon bundle, its statstable (code_post_gfx_mp) for the families, and
/// the attachment configurations prepared from those.
fn load_iw5_guest_check(progress: &LoadProgress) -> GuestCheckLoad {
    let mut report = Vec::new();
    let empty = |report| GuestCheckLoad {
        weapons: WeaponBuild::default(),
        fpv_names: Default::default(),
        xanims: XAnimBuild::default(),
        report,
    };
    let Some(donor) = resolve_iw5_weapon_donor(None, &mut report) else {
        return empty(report);
    };
    let job = load_jobs::open(JobKind::ImageDecode).namespace("iw5");
    let (bundle, _images, walked) = walk_iw5_weapon_bundle(&donor, progress, job);
    report.extend(walked);
    let stats = match find_zone_for_tree(&donor, "code_post_gfx_mp") {
        Ok(found) => walk_population_file(&found.path, progress, MaterialCatalog::default()).cac_tables,
        Err(error) => {
            report.push(format!("iw5 statstable: {error}"));
            Vec::new()
        }
    };
    let mut weapons = bundle.weapons;
    weapons.apply_stats_tables(&stats);
    weapons.set_family_tables(
        stats
            .into_iter()
            .chain(bundle.stats_tables)
            .map(|table| (asset_core::AssetNamespace::Iw5, table))
            .collect(),
    );
    let prepared = weapons.prepare_iw5_configurations();
    report.push(format!(
        "iw5 configurations: prepared={} refused={}",
        prepared.prepared,
        prepared.refused.len()
    ));
    let fpv_names = (0..bundle.fpv.len()).filter_map(|i| bundle.fpv.name_at(i).map(str::to_owned)).collect();
    GuestCheckLoad { weapons, fpv_names, xanims: bundle.xanims, report }
}

fn check_guest_game(game: &str, ns: asset_core::AssetNamespace) -> Vec<String> {
    let progress = LoadProgress::default();
    let GuestCheckLoad { mut weapons, fpv_names, xanims, report } = load_guest_check(game, ns, &progress);
    let mut out: Vec<String> = report.iter().map(|l| format!("  {l}")).collect();
    weapons.set_guest_names(true);
    let (mut ok, mut bad) = (0, 0);
    let listed: Vec<&bo2_profile::guest::GuestGun> = bo2_profile::guest::all().filter(|g| g.game == game).collect();
    for gun in &listed {
        let key = asset_core::AssetKey::new(ns, asset_core::AssetKind::Weapon, format!("{}_mp", gun.gun));
        let Some(id) = key.ok().and_then(|k| weapons.resolve_key(&k).ok().flatten()) else {
            bad += 1;
            let near: Vec<String> = (1..=weapons.len() as u32)
                .filter(|&i| weapons.name_of(i).starts_with(gun.gun))
                .take(3)
                .map(|i| format!("{}:{:?}", weapons.name_of(i), weapons.namespace_of(i)))
                .collect();
            out.push(format!("{:<22} MISSING: no `{}_mp` in its game's common_mp (near: {})", gun.reference(), gun.gun, near.join(", ")));
            continue;
        };
        let script = weapons.script_name_of(id);
        let base = script.split('+').next().unwrap_or(&script).to_owned();
        let variant_names: Vec<String> = (1..=weapons.len() as u32)
            .map(|i| weapons.script_name_of(i))
            .filter(|n| n.starts_with(&format!("{base}+")))
            .collect();
        let variants = variant_names.len();
        let gun_model = weapons.gun_xmodel_of(id).unwrap_or("").to_owned();
        let fpv = !gun_model.is_empty() && fpv_names.contains(&gun_model);
        let world = weapons.world_model_of(id).unwrap_or("").to_owned();
        let anims = weapons.sz_xanims_of(id).map_or(0, |a| a.iter().filter(|x| x.is_some()).count());
        // (Named animations its zone does not hold: they never play.)
        let anims_absent: Vec<String> = weapons.sz_xanims_of(id).map_or(Vec::new(), |a| {
            a.iter()
                .flatten()
                .filter(|n| !n.is_empty() && xanims.get(ns, n).is_none())
                .cloned()
                .collect()
        });
        // (Arms are BO2's in a match: `resolve_fpv_hands` gives every gun
        // BO2's when BO2's guns are there, so no hands gap counts here.)
        let gaps: Vec<String> = weapons
            .dependency_gaps_of(id)
            .into_iter()
            .filter(|g| !g.kind.contains("hands"))
            .map(|g| format!("{} `{}`", g.kind, g.name))
            .collect();
        let admission = if !weapons.configuration_supported(id) {
            "REFUSED (dual-wield model)".to_owned()
        } else if gaps.is_empty() {
            "ok".to_owned()
        } else {
            format!("MISSING {}", gaps.join(", "))
        };
        let fine = fpv && !world.is_empty() && anims > 0 && admission == "ok";
        if fine {
            ok += 1;
        } else {
            bad += 1;
        }
        out.push(format!(
            "{:<22} {} script={script} variants={variants}{} view={gun_model}{} world={} anims={anims}{} admission={admission}",
            gun.reference(),
            if fine { "OK  " } else { "GAPS" },
            variant_names.first().map_or(String::new(), |v| format!(" (e.g. {v})")),
            if fpv { "" } else { " (NO FIRST-PERSON MESH)" },
            if world.is_empty() { "-" } else { world.as_str() },
            if anims_absent.is_empty() {
                String::new()
            } else {
                format!(" (not in zone: {})", anims_absent.iter().take(4).cloned().collect::<Vec<_>>().join(" "))
            },
        ));
    }
    out.push(format!("{game}: {ok} guest guns OK, {bad} with gaps"));
    // The game's other guns with their attachment words (what could be
    // listed next).
    let mut others: Vec<String> = (1..=weapons.len() as u32)
        .filter(|&i| weapons.namespace_of(i) == Some(ns))
        .map(|i| weapons.script_name_of(i))
        .filter(|n| !n.contains('+'))
        .filter(|n| !listed.iter().any(|g| format!("{}_mp", g.reference()) == *n))
        .filter(|n| {
            (1..=weapons.len() as u32).any(|i| weapons.script_name_of(i).starts_with(&format!("{n}+")))
                || weapons.resolve_index(n.trim_start_matches(ns.guest_prefix())).ok().flatten().and_then(|i| weapons.facts_of(i)).is_some_and(|f| f.weap_class <= 5)
        })
        .map(|n| {
            let mut words: Vec<String> = (1..=weapons.len() as u32)
                .map(|i| weapons.script_name_of(i))
                .filter(|v| v.starts_with(&format!("{n}+")))
                .flat_map(|v| v.split('+').skip(1).map(str::to_owned).collect::<Vec<_>>())
                .collect();
            words.sort();
            words.dedup();
            format!("{n} [{}]", words.join(" "))
        })
        .collect();
    others.sort();
    out.push(format!("{game}: other guns: {}", others.join("; ")));
    out
}
