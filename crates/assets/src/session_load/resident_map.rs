use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ZoneStamp {
    pub(super) path: PathBuf,
    pub(super) len: u64,
    modified: Option<std::time::SystemTime>,
}

impl ZoneStamp {
    pub(super) fn of(path: &Path) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        Some(Self {
            path: path.to_path_buf(),
            len: meta.len(),
            modified: meta.modified().ok(),
        })
    }
}

struct ResidentMap {
    pub(super) zone: ZoneStamp,
    pub(super) common: Arc<CommonSet>,
    pub(super) prepared: PreparedMatch,
    /// Its decoded images were let go once the card had them all
    /// ([`release_resident_cpu_images`]): it can stand in for a walk only
    /// while the card still holds that walk's images.
    stripped: bool,
}

static RESIDENT_MAP: std::sync::Mutex<Option<ResidentMap>> = std::sync::Mutex::new(None);

/// The walk whose map images the card holds now (0: none). The render
/// frontend says so when a map's upload completes and when it lets them go.
static GPU_RESIDENT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn note_gpu_resident(products_id: u64) {
    GPU_RESIDENT.store(products_id, std::sync::atomic::Ordering::Relaxed);
}

/// The card has every image of walk `products_id`: the copy kept for a
/// same-map load lets go of its decoded images (each one is the texture's
/// bytes a second time, ~580 MB on Nuketown; 8 games at once on one PC
/// cannot carry that). Returns how many it let go and the bytes that were
/// actually freed (an image the shared common set also holds stays).
pub fn release_resident_cpu_images(products_id: u64) -> Option<(usize, u64)> {
    let mut slot = RESIDENT_MAP
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let resident = slot.as_mut()?;
    if resident.stripped || resident.prepared.materials.products_id != products_id {
        return None;
    }
    let held = Arc::strong_count(&resident.prepared.materials.population);
    let Some(population) = Arc::get_mut(&mut resident.prepared.materials.population) else {
        diag::info!(
            World,
            "resident map: walk #{products_id} keeps its decoded images (its materials are held {held} times)"
        );
        return None;
    };
    let (mut count, mut freed) = (0, 0u64);
    for image in &mut population.images {
        if let Some(decoded) = image.decoded.take() {
            count += 1;
            if Arc::strong_count(&decoded) == 1 {
                freed += decoded.data.as_ref().map_or(0, |data| data.len() as u64);
            }
        }
    }
    resident.stripped = true;
    Some((count, freed))
}

fn resident_copy(zone: &ZoneStamp, key: &CommonKey) -> Option<PreparedMatch> {
    let common = landed_common(key)?;
    let slot = RESIDENT_MAP
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let resident = slot.as_ref()?;
    // Without its decoded images the copy is good only for a load the card
    // already has every texture of.
    if resident.stripped
        && GPU_RESIDENT.load(std::sync::atomic::Ordering::Relaxed)
            != resident.prepared.materials.products_id
    {
        return None;
    }
    (resident.zone == *zone && Arc::ptr_eq(&resident.common, &common))
        .then(|| resident.prepared.clone())
}

pub async fn load_prepared_match(
    zone_ff: Result<PathBuf, String>,
    common_mp: Result<PathBuf, String>,
    progress: LoadProgress,
) -> MatchLoadOutcome {
    let stamp = zone_ff.as_deref().ok().and_then(ZoneStamp::of);
    if let Some(stamp) = &stamp {
        let key = CommonKey::for_match(
            Some(&stamp.path),
            common_mp.as_deref().ok(),
            &mut Vec::new(),
        );
        let copying = std::time::Instant::now();
        if let Some(mut prepared) = resident_copy(stamp, &key) {
            progress.record_reused_scoped(StageId::CommonAssets, "shared common");
            progress.record_reused_scoped(StageId::MapAssets, "resident");
            progress.record_reused_scoped(StageId::Images, "map");
            let line = format!(
                "resident map: reused `{}` walk #{} — no zone opened, no decode; match copy {:.0}ms",
                stamp.path.display(),
                prepared.materials.products_id,
                copying.elapsed().as_secs_f32() * 1000.0
            );
            diag::info!(World, "{line}");
            prepared.report.push(line);
            prepared.report.extend(progress.timing_report());
            return MatchLoadOutcome::Ready(prepared);
        }
    }
    let dropped = RESIDENT_MAP
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .take();
    if let Some(dropped) = dropped {
        diag::info!(
            World,
            "resident map: `{}` walk #{} dropped before the next walk",
            dropped.zone.path.display(),
            dropped.prepared.materials.products_id
        );
    }
    let (outcome, common) = walk_prepared_match(zone_ff, common_mp, progress.clone()).await;
    let MatchLoadOutcome::Ready(mut prepared) = outcome else {
        return outcome;
    };
    if let (Some(zone), Some(common)) = (stamp, common) {
        let keeping = std::time::Instant::now();
        let resident = prepared.clone();
        prepared.report.push(format!(
            "resident map: kept `{}` walk #{} for a same-map load ({:.0}ms)",
            zone.path.display(),
            prepared.materials.products_id,
            keeping.elapsed().as_secs_f32() * 1000.0
        ));
        *RESIDENT_MAP
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(ResidentMap {
            zone,
            common,
            prepared: resident,
            stripped: false,
        });
    }
    prepared.report.extend(progress.timing_report());
    MatchLoadOutcome::Ready(prepared)
}
