//! bo2zm: readiness checks for a Black Ops II map, read-only.
//! `t6check [BO2 folder]` walks the nine Nuketown Zombies zones and checks
//! that every image they use has pixels: in the zone or in a pack, decoding
//! to an IWI whose size matches the zone's description of it.
//! `t6check <BO2 folder> <zone>...` checks those zones instead (found in the
//! install or the `IW4L_T6_EXTRA` folders) and lists every missing image.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use asset_t6::{ImageSource, PackSet, capture_zone, legacy_images};

const DEFAULT_BO2: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Call of Duty Black Ops II";

const NUKETOWN_ZONES: [&str; 9] = [
    "code_pre_gfx_zm",
    "code_post_gfx_zm",
    "common_zm",
    "patch_zm",
    "ui_zm",
    "patch_ui_zm",
    "dlczm0_load_zm",
    "zm_nuked",
    "zm_nuked_patch",
];

fn main() -> ExitCode {
    let root = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from(DEFAULT_BO2), PathBuf::from);
    let zones: Vec<String> = std::env::args().skip(2).collect();
    match run(&root, &zones) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("t6check: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(root: &Path, zones: &[String]) -> Result<bool, String> {
    let started = Instant::now();
    let dir = root.join("zone").join("all");
    let packs = PackSet::open_dir(&dir)?;
    println!(
        "packs: {}",
        packs
            .packs
            .iter()
            .map(|p| format!("{} ({})", p.name, p.len()))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let legacy = Legacy::open();
    if !legacy.mains.is_empty() {
        println!(
            "other games' IWDs: {}",
            legacy.mains.iter().map(|(m, _, _)| m.display().to_string()).collect::<Vec<_>>().join(", ")
        );
    }

    let mut ok = true;
    let mut images_total = 0usize;
    let mut by_source: BTreeMap<String, usize> = BTreeMap::new();
    let mut missing = Vec::new();
    let mut size_mismatch = Vec::new();
    let mut decode_fail = Vec::new();
    let names: Vec<&str> = if zones.is_empty() {
        NUKETOWN_ZONES.to_vec()
    } else {
        zones.iter().map(String::as_str).collect()
    };
    let all = !zones.is_empty();
    for zone in names {
        let path = asset_transport::t6_extra::file_in(&dir, &format!("{zone}.ff"));
        let capture = capture_zone(&path)?;
        println!("{zone}: {} images", capture.images.len());
        if std::env::var_os("T6CHECK_DUMP").is_some() {
            dump(&capture);
        }
        for image in &capture.images {
            images_total += 1;
            match packs.locate(image) {
                Some(ImageSource::Pack(i, entry)) => {
                    let pack = &packs.packs[i];
                    // Found by its name alone (another build of the
                    // picture): its size may differ; that is no failure.
                    let by_name = pack.find(image.name_hash, image.part_hash).is_none();
                    let label = if by_name { format!("{} (by name)", pack.name) } else { pack.name.clone() };
                    *by_source.entry(label).or_default() += 1;
                    match pack.read(entry).and_then(|bytes| {
                        ipak_t6::parse_iwi(&bytes)
                            .map(|iwi| (iwi.width, iwi.height))
                            .map_err(|e| e.to_string())
                    }) {
                        Ok((w, h)) => {
                            if !by_name && (w, h) != (image.width as u32, image.height as u32) {
                                size_mismatch.push(format!(
                                    "{} zone {}x{} pack {w}x{h}",
                                    image.name, image.width, image.height
                                ));
                            }
                        }
                        Err(e) => decode_fail.push(format!("{}: {e}", image.name)),
                    }
                }
                Some(ImageSource::Embedded) => {
                    *by_source.entry("in the zone".into()).or_default() += 1
                }
                Some(ImageSource::Empty) => {
                    *by_source
                        .entry("no pixels (runtime image)".into())
                        .or_default() += 1
                }
                None => match legacy.find(&image.name) {
                    Some(from) => *by_source.entry(format!("{from} IWDs")).or_default() += 1,
                    None => missing.push(format!(
                        "{} ({zone}) parts={} hash={:08x}/{:08x}{}",
                        image.name,
                        image.streamed_parts,
                        image.name_hash,
                        image.part_hash,
                        if legacy_images::is_normal_map_name(&image.name) { " (normal map)" } else { "" }
                    )),
                },
            }
        }
    }
    println!("images: {images_total}");
    for (k, n) in &by_source {
        println!("   {n:>6}  {k}");
    }
    for (label, list) in [
        ("MISSING", &missing),
        ("SIZE MISMATCH", &size_mismatch),
        ("DECODE FAILED", &decode_fail),
    ] {
        if !list.is_empty() {
            ok = false;
            println!("{label}: {}", list.len());
            for item in list.iter().take(if all { usize::MAX } else { 12 }) {
                println!("   {item}");
            }
        }
    }
    println!(
        "{} in {:.2?}",
        if ok {
            "ALL IMAGES OK"
        } else {
            "IMAGE CHECK FAILED"
        },
        started.elapsed()
    );
    Ok(ok)
}

/// `T6CHECK_DUMP=1`: every model's materials and every material's
/// technique set, sort key and images, one line each.
fn dump(c: &asset_t6::ZoneCapture) {
    let mat = |k: &Option<asset_t6::AssetKey>| {
        k.and_then(|k| c.materials.get(k.index))
            .map_or("?", |m| m.name.as_str())
    };
    for i in &c.images {
        if let Some(e) = &i.embedded {
            println!(
                "  embedded {} {}x{} dxgi={} levels={} flags={:#x} bytes={}",
                i.name,
                i.width,
                i.height,
                e.dxgi_format,
                e.level_count,
                e.flags,
                e.data.len()
            );
        }
    }
    for m in &c.xmodels {
        let mats: Vec<&str> = m.materials.iter().map(mat).collect();
        println!("  xmodel {} [{}]", m.name, mats.join(" "));
    }
    for m in &c.materials {
        let ts = m
            .technique_set
            .and_then(|k| c.technique_sets.get(k.index))
            .map_or("?", |t| t.name.as_str());
        let imgs: Vec<String> = m
            .textures
            .iter()
            .map(|t| {
                let n = t
                    .image
                    .and_then(|k| c.images.get(k.index))
                    .map_or("?", |i| i.name.as_str());
                format!("{}:{n}", t.semantic)
            })
            .collect();
        println!(
            "  material {} ts={ts} sort={} [{}]",
            m.name,
            m.sort_key,
            imgs.join(" ")
        );
    }
}

/// Black Ops' and World at War's IWDs (`IW4L_GAMES`), where the engine
/// finds pictures a Black Ops II zone names but no Black Ops II pack holds
/// (`asset_t6::legacy_images`): each `main`, its index and sorted names.
struct Legacy {
    mains: Vec<(PathBuf, std::sync::Arc<asset_transport::IwdIndex>, Vec<String>)>,
}

impl Legacy {
    fn open() -> Legacy {
        let mains = legacy_images::legacy_iwd_mains()
            .iter()
            .filter_map(|m| {
                let index = asset_transport::IwdIndex::open(m).ok()?;
                let mut names: Vec<String> = index.image_names().map(str::to_owned).collect();
                names.sort();
                Some((m.clone(), index, names))
            })
            .collect();
        Legacy { mains }
    }

    /// The game folder whose IWDs hold `name` as the engine looks for it.
    fn find(&self, name: &str) -> Option<String> {
        if !legacy_images::wants_legacy(name) {
            return None;
        }
        let game = |m: &Path| m.parent().and_then(|p| p.file_name()).map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        self.mains
            .iter()
            .find(|(_, index, _)| legacy_images::legacy_names(name).iter().any(|c| index.image_candidates(c).is_some()))
            .or_else(|| {
                self.mains.iter().find(|(_, index, names)| {
                    legacy_images::legacy_cut_name(name, names).is_some_and(|c| index.image_candidates(&c).is_some())
                })
            })
            .map(|(m, _, _)| game(m))
    }
}
