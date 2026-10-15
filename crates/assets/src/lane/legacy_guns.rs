//! bo2mp: Call of Duty 4's and World at War's guns as guest guns in a Black
//! Ops II match (`cod4_ak47_mp+reflex`, `waw_mp40_mp+silencer`), from each
//! game's `common_mp`.
//!
//! The `iw3` and `t4` readers load the zone (every asset: both games'
//! common zones read to the end); World at War's models, materials and
//! images are converted to CoD4's types (`t4::convert`), so one path builds
//! both games' guns the way Ghosts' go in (`super::iw6`):
//!
//! - models into Black Ops II's captured form (`asset_t6::XModelRef`) and
//!   through its skinned-model builder: CoD4's packed vertex is IW4's
//!   (colour B,G,R,A, texture coordinates u in the high half, unit vectors
//!   as biased bytes), repacked as BO2's; the skeleton and the blend and
//!   rigid vertex streams are the same;
//! - animations into BO2's captured form (the same IW-family part types and
//!   packing) under the game's namespace;
//! - materials onto BO2's draw path (`T6Draw` from the technique set's
//!   name, as the CoD4 map lane draws world surfaces) with their colour
//!   maps from the game's own IWDs, kept small;
//! - weapons through `WeaponCatalog::capture_legacy`, read by member name
//!   through each reader's schema.
//!
//! Arms are Black Ops II's (the catalog's hands resolve to BO2's). World
//! at War's guns' sounds are its own: the sound bank reads its common
//! zone's aliases (`asset_audio::waw_weapon_sounds`, under the World at War
//! namespace the guns carry), taken from this read when it comes first.
//! CoD4's are not read yet (the `iw3` reader has no sound aliases).

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use asset_anim::XAnimBuild;
use asset_core::{AssetEdge, AssetNamespace, AssetRef, T6Blend, T6Cull, T6Draw, ZoneOwner};
use asset_game::{LegacyGame, LegacyWeaponView, WeaponBuild, WeaponCatalog};
use asset_material::{AuthoredImage, AuthoredMaterial, MaterialCatalog, MaterialTextureBinding, TS_COLOR_MAP};
use asset_model::{FpvMeshBuild, WorldWeaponBuild};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// The largest a first-person gun's colour map is kept, and a third-person
/// one's (`BO2MP_LEGACY_TEXTURE=<size>` changes the first).
const FIRST_PERSON_MAX: u32 = 512;
const WORLD_MAX: u32 = 128;

/// The techniques whose draw state a model material takes, first present
/// wins: lit with sun shadow, lit with sun, lit, emissive, unlit (CoD4's
/// numbering; World at War's are converted to it).
const TECHNIQUES: [usize; 5] = [9, 8, 7, 5, 4];

/// Which game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Legacy {
    /// Call of Duty 4.
    Iw3,
    /// World at War.
    T4,
}

impl Legacy {
    pub(crate) fn namespace(self) -> AssetNamespace {
        match self {
            Self::Iw3 => AssetNamespace::Iw3,
            Self::T4 => AssetNamespace::T4,
        }
    }

    /// The `game` of its guest guns (`bo2_profile::guest`).
    pub(crate) fn tag(self) -> &'static str {
        match self {
            Self::Iw3 => "iw3",
            Self::T4 => "t4",
        }
    }

    fn version(self) -> u32 {
        match self {
            Self::Iw3 => asset_transport::discover::IW3_ZONE_VERSION,
            Self::T4 => asset_transport::discover::T4_ZONE_VERSION,
        }
    }
}

/// A game's guns, ready to join the match's catalogs.
pub(crate) struct LegacyGuests {
    pub weapons: WeaponBuild,
    pub fpv: FpvMeshBuild,
    pub world_guns: WorldWeaponBuild,
    pub xanims: XAnimBuild,
    pub material_seed: MaterialCatalog,
    pub report: Vec<String>,
}

impl LegacyGuests {
    pub(crate) fn empty(material_seed: MaterialCatalog, report: Vec<String>) -> Self {
        Self {
            weapons: WeaponBuild::default(),
            fpv: FpvMeshBuild::default(),
            world_guns: WorldWeaponBuild::default(),
            xanims: XAnimBuild::default(),
            material_seed,
            report,
        }
    }
}

/// The game's `common_mp`, if it is installed.
pub(crate) fn legacy_common_mp(game: Legacy) -> Result<PathBuf, String> {
    let root = asset_transport::games_root_from_env()?;
    asset_transport::find_common_mp_for_envelope(&root, game.version()).map(|zone| zone.path)
}

/// A game's `common_mp` read ahead (the slow part, which needs no
/// material pool): `build_legacy_guests` then links it into the match's.
pub(crate) struct LegacyPrefetch {
    game: Legacy,
    read: Result<(PathBuf, Arc<LegacyRead>), String>,
}

/// bo2mp: read the game's guns for the match (`wanted`: Black Ops II rules
/// with guest guns on), or say why not. A reader's panic is a gap, never a
/// lost match.
pub(crate) fn prefetch_legacy_guest_guns(game: Legacy, wanted: bool) -> LegacyPrefetch {
    if !wanted {
        return LegacyPrefetch { game, read: Err("off".into()) };
    }
    let read = legacy_common_mp(game).and_then(|path| {
        let read = std::panic::catch_unwind(|| read_legacy_shared(game, &path))
            .unwrap_or_else(|_| Err(format!("reading {} panicked", path.display())));
        read.map(|read| (path, read))
    });
    LegacyPrefetch { game, read }
}

/// bo2mp: the match's CoD4 or World at War guest guns from what
/// `prefetch_legacy_guest_guns` read, their materials linked into
/// `material_seed`.
pub(crate) fn build_legacy_guest_guns(
    prefetch: LegacyPrefetch,
    material_seed: MaterialCatalog,
    progress: &asset_transport::LoadProgress,
) -> LegacyGuests {
    let LegacyPrefetch { game, read } = prefetch;
    let tag = game.tag();
    let (path, read) = match read {
        Ok(read) => read,
        Err(e) => return LegacyGuests::empty(material_seed, vec![format!("{tag} guest guns: {e}")]),
    };
    let stage = progress.begin_scoped(asset_transport::StageId::CommonAssets, format!("{tag}_guest_weapons"), None);
    let seed_on_panic = material_seed.clone();
    let guests = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build_legacy_guests(game, &path, read, material_seed)))
        .unwrap_or_else(|_| {
            LegacyGuests::empty(seed_on_panic, vec![format!("{tag} guest guns: building {} panicked", path.display())])
        });
    stage.done();
    guests
}

/// What a game's `common_mp` gives: its models, materials and images as
/// CoD4's types, its weapons and its animations.
struct LegacyRead {
    zone: iw3::zone::Zone,
    weapons: Vec<LegacyWeaponView>,
    anims: Vec<asset_t6::XAnimRef>,
    walked: String,
}

fn u16s(b: &[u8]) -> Vec<u16> {
    b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

fn u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

/// The two readers' schema-loaded nodes have the same shape (`t4`'s is a
/// copy of `iw3`'s), so one body reads both, with the reader's accessors
/// (`access`) in scope; `$schema` is its schema, `$name` an asset's name by
/// id.
macro_rules! legacy_reads {
    ($game:expr, $schema:expr, $strings:expr, $name:expr, $weapons:expr, $anims:expr) => {{
        let strings: &[String] = $strings;
        let script = |i: u16| strings.get(usize::from(i)).cloned().unwrap_or_default();
        let raw = |n: &_, field: &str| -> Vec<u8> {
            let n: &_ = n;
            match n_field(n, field) {
                Some(V::Bytes(b)) => b.clone(),
                Some(V::Nodes(nodes)) => nodes.iter().flat_map(|c| c.data.iter().copied()).collect(),
                _ => Vec::new(),
            }
        };
        let schema = $schema;
        let def = schema.types.get("WeaponDef");
        let offsets: HashMap<String, usize> = def
            .map(|t| {
                t.members
                    .iter()
                    .filter_map(|m| Some((m.name.clone()?, m.offset as usize)))
                    .collect()
            })
            .unwrap_or_default();
        let counts: HashMap<String, usize> = def
            .map(|t| {
                t.members
                    .iter()
                    .filter_map(|m| Some((m.name.clone()?, m.arr.first().copied().unwrap_or(1) as usize)))
                    .collect()
            })
            .unwrap_or_default();
        let weapon = |root: &_| -> LegacyWeaponView {
            let root: &_ = root;
            let name_of = |id: Option<usize>| id.map($name).filter(|n: &String| !n.is_empty());
            let u16_at = |at: usize| root_data(root).get(at..at + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
            let script_list = |member: &str| -> Vec<String> {
                let (Some(&at), Some(&n)) = (offsets.get(member), counts.get(member)) else { return Vec::new() };
                (0..n).map(|k| script(u16_at(at + k * 2))).collect()
            };
            let mut sounds = HashMap::new();
            let mut bounce = Vec::new();
            let mut assets = HashMap::new();
            for (field, value) in n_fields(root) {
                match value {
                    V::Nodes(nodes) if nodes.first().is_some_and(|c| c.ty == "SndAliasCustom") => {
                        let names: Vec<String> = nodes
                            .iter()
                            .map(|c| {
                                n_nodes(c, "name")
                                    .first()
                                    .and_then(|n| n_string(n, "soundName"))
                                    .unwrap_or("")
                                    .to_owned()
                            })
                            .collect();
                        if field == "bounceSound" {
                            bounce = names;
                        } else if let Some(alias) = names.into_iter().next() {
                            sounds.insert(field.clone(), alias);
                        }
                    }
                    V::Asset(Some(id)) => {
                        if let Some(n) = name_of(Some(*id)) {
                            assets.insert(field.clone(), n);
                        }
                    }
                    _ => {}
                }
            }
            let first = |field: &str| n_assets(root, field).first().copied().flatten();
            let keys = script_list("notetrackSoundMapKeys");
            let values = script_list("notetrackSoundMapValues");
            LegacyWeaponView {
                game: $game,
                name: n_string(root, "szInternalName").unwrap_or("").to_owned(),
                display_name: n_string(root, "szDisplayName").unwrap_or("").to_owned(),
                def: root_data(root).to_vec(),
                offsets: offsets.clone(),
                gun_model: name_of(first("gunXModel")),
                world_model: name_of(first("worldModel")),
                xanims: n_strings(root, "szXAnims").into_iter().map(|s| s.unwrap_or("").to_owned()).collect(),
                hide_tags: script_list("hideTags").into_iter().filter(|t| !t.is_empty()).collect(),
                alt_weapon_name: n_string(root, "szAltWeaponName").unwrap_or("").to_owned(),
                sounds,
                bounce,
                assets,
                notetrack_sounds: keys
                    .into_iter()
                    .zip(values)
                    .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                    .collect(),
            }
        };
        let anim = |root: &_| -> Option<asset_t6::XAnimRef> {
            let root: &_ = root;
            let name = n_string(root, "name")?.to_owned();
            let numframes = n_int(root, "numframes") as u16;
            let indices = n_nodes(root, "indices")
                .first()
                .map(|ix| {
                    if numframes < 256 {
                        raw(ix, "_1").into_iter().map(u16::from).collect()
                    } else {
                        u16s(&raw(ix, "_2"))
                    }
                })
                .unwrap_or_default();
            Some(asset_t6::XAnimRef {
                delta_quat: Vec::new(),
                name,
                numframes,
                framerate: n_float(root, "framerate"),
                looping: n_int(root, "bLoop") != 0,
                delta: false,
                delta_trans: Vec::new(),
                bone_count: std::array::from_fn(|k| n_int(root, &format!("boneCount[{k}]")) as u8),
                names: u16s(&raw(root, "names")).into_iter().map(script).collect(),
                notifies: n_nodes(root, "notify")
                    .iter()
                    .map(|c| {
                        let d = &c.data;
                        let id = d.get(0..2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
                        let time = d.get(4..8).map_or(0.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                        (script(id), time)
                    })
                    .collect(),
                data_byte: raw(root, "dataByte"),
                data_short: u16s(&raw(root, "dataShort")),
                data_int: u32s(&raw(root, "dataInt")),
                random_data_byte: raw(root, "randomDataByte"),
                random_data_short: u16s(&raw(root, "randomDataShort")),
                random_data_int: u32s(&raw(root, "randomDataInt")),
                indices,
            })
        };
        let weapons: Vec<LegacyWeaponView> = $weapons.map(|root| weapon(root)).collect();
        let anims: Vec<asset_t6::XAnimRef> = $anims.filter_map(|root| anim(root)).collect();
        (weapons, anims)
    }};
}

/// The reader-specific accessors `legacy_reads!` calls (the node types are
/// different types with the same methods).
mod access {
    macro_rules! accessors {
        () => {
            pub fn n_field<'a>(n: &'a N, f: &str) -> Option<&'a V> {
                n.field(f)
            }
            pub fn n_fields(n: &N) -> &[(String, V)] {
                &n.fields
            }
            pub fn n_nodes<'a>(n: &'a N, f: &str) -> &'a [N] {
                n.nodes(f)
            }
            pub fn n_string<'a>(n: &'a N, f: &str) -> Option<&'a str> {
                n.string(f)
            }
            pub fn n_strings<'a>(n: &'a N, f: &str) -> Vec<Option<&'a str>> {
                n.strings(f)
            }
            pub fn n_assets(n: &N, f: &str) -> Vec<Option<usize>> {
                n.assets(f)
            }
            pub fn n_int(n: &N, f: &str) -> i64 {
                n.int(f)
            }
            pub fn n_float(n: &N, f: &str) -> f32 {
                n.float(f)
            }
            pub fn root_data(n: &N) -> &[u8] {
                &n.data
            }
        };
    }
    pub(super) mod iw3n {
        pub use iw3::zone::generic::{GNode as N, GVal as V};
        accessors!();
    }
    pub(super) mod t4n {
        pub use t4::zone::{GNode as N, GVal as V};
        accessors!();
    }
}

fn read_iw3(path: &Path) -> Result<LegacyRead, String> {
    use access::iw3n::*;
    let data = iw3::fastfile::load(path).map_err(|e| format!("{e:#}"))?;
    let zone = iw3::zone::Zone::parse(&data, iw3::zone::ParseOptions::default()).map_err(|e| format!("{e:#}"))?;
    let walked = format!(
        "{} of {} assets{}",
        zone.stats.top_level_parsed,
        zone.stats.top_level_total,
        zone.stats.stopped_at.as_ref().map_or_else(String::new, |s| format!(", stopped at {s}"))
    );
    let (weapons, anims) = {
        let names = |id: usize| zone.assets.get(id).map(|a| a.name().to_owned()).unwrap_or_default();
        let of = |ty: iw3::zone::AssetType| {
            zone.assets.iter().filter_map(move |a| match a {
                iw3::zone::Asset::Generic(g) if g.ty == ty => Some(&g.root),
                _ => None,
            })
        };
        legacy_reads!(
            LegacyGame::Iw3,
            iw3::zone::schema::schema(),
            &zone.script_strings,
            names,
            of(iw3::zone::AssetType::Weapon),
            of(iw3::zone::AssetType::XAnimParts)
        )
    };
    Ok(LegacyRead { zone, weapons, anims, walked })
}

fn read_t4(path: &Path) -> Result<LegacyRead, String> {
    use access::t4n::*;
    let data = t4::fastfile::load(path).map_err(|e| format!("{e:#}"))?;
    let parsed = t4::zone::Zone::parse(&data, t4::zone::ParseOptions::default()).map_err(|e| format!("{e:#}"))?;
    let walked = format!(
        "{} of {} assets{}",
        parsed.stats.top_level_parsed,
        parsed.stats.top_level_total,
        parsed.stats.stopped_at.as_ref().map_or_else(String::new, |s| format!(", stopped at {s}"))
    );
    let (weapons, anims) = {
        let names = |id: usize| parsed.assets.get(id).map(|a| a.name.clone()).unwrap_or_default();
        legacy_reads!(
            LegacyGame::T4,
            t4::zone::schema::schema(),
            &parsed.script_strings,
            names,
            parsed.of_type(t4::zone::AssetType::Weapon).map(|(_, a)| &a.root),
            parsed.of_type(t4::zone::AssetType::XAnimParts).map(|(_, a)| &a.root)
        )
    };
    // The guns' sounds, from this read (the sound bank then needs no read
    // of its own).
    if crate::guest_guns() {
        asset_audio::waw_sounds_from_zone(path, &parsed);
    }
    let zone = t4::convert::to_iw3_map(&parsed).zone;
    Ok(LegacyRead { zone, weapons, anims, walked })
}

/// bo2mp: one read of a game's `common_mp` shared by whoever needs it at
/// once (the guest guns, the map's props and their materials): the second
/// to ask waits for the first's read. Kept only while someone holds it.
fn read_legacy_shared(game: Legacy, path: &Path) -> Result<Arc<LegacyRead>, String> {
    use std::sync::{Mutex, Weak};
    static SHARED: Mutex<Vec<(Legacy, PathBuf, Weak<LegacyRead>)>> = Mutex::new(Vec::new());
    static READING: Mutex<()> = Mutex::new(());
    let _one_at_a_time = READING.lock().unwrap_or_else(|e| e.into_inner());
    let find = || {
        SHARED
            .lock()
            .ok()?
            .iter()
            .find(|(g, p, _)| *g == game && p == path)
            .and_then(|(_, _, w)| w.upgrade())
    };
    if let Some(read) = find() {
        return Ok(read);
    }
    let read = Arc::new(read_legacy(game, path)?);
    if let Ok(mut s) = SHARED.lock() {
        s.retain(|(_, _, w)| w.strong_count() > 0);
        s.push((game, path.to_path_buf(), Arc::downgrade(&read)));
    }
    Ok(read)
}

/// bo2mp: a game's `common_mp` zone, for a map's references into it (the
/// materials and models a map zone names but does not hold).
pub(crate) fn legacy_common_zone(game: Legacy) -> Option<Arc<impl std::ops::Deref<Target = iw3::zone::Zone>>> {
    let path = legacy_common_mp(game).ok()?;
    read_legacy_shared(game, &path).ok().map(|r| Arc::new(CommonZone(r)))
}

/// The zone of a shared `common_mp` read.
pub(crate) struct CommonZone(Arc<LegacyRead>);

impl std::ops::Deref for CommonZone {
    type Target = iw3::zone::Zone;
    fn deref(&self) -> &iw3::zone::Zone {
        &self.0.zone
    }
}

fn read_legacy(game: Legacy, path: &Path) -> Result<LegacyRead, String> {
    let started = std::time::Instant::now();
    let mut read = match game {
        Legacy::Iw3 => read_iw3(path),
        Legacy::T4 => read_t4(path),
    }
    .map_err(|e| format!("read {}: {e}", path.display()))?;
    read.walked.push_str(&format!(" in {:.1} s", started.elapsed().as_secs_f64()));
    Ok(read)
}

/// A game's guns from its `common_mp`, their materials linked into
/// `material_seed` (read and built in one go: `iw4l check-guns`).
pub(crate) fn load_legacy_guests(game: Legacy, path: &Path, material_seed: MaterialCatalog) -> LegacyGuests {
    match read_legacy(game, path) {
        Ok(read) => build_legacy_guests(game, path, Arc::new(read), material_seed),
        Err(e) => LegacyGuests::empty(material_seed, vec![format!("{} guest guns: {e}", game.tag())]),
    }
}

fn build_legacy_guests(game: Legacy, path: &Path, read: Arc<LegacyRead>, material_seed: MaterialCatalog) -> LegacyGuests {
    let started = std::time::Instant::now();
    let tag = game.tag();
    let ns = game.namespace();
    let mut report = Vec::new();
    report.push(format!(
        "{tag} guest guns: {} read ({}): {} weapons, {} animations",
        path.display(),
        read.walked,
        read.weapons.len(),
        read.anims.len()
    ));
    let main = asset_transport::game_root_for_zone(path).ok().map(|root| root.join("main"));
    let mut b = Builder {
        zone: &read.zone,
        commons: Vec::new(),
        common_names: None,
        tag,
        ns,
        main,
        catalog: material_seed,
        owner: ZoneOwner::from_zone_path(path),
        materials: HashMap::new(),
        images: HashMap::new(),
        decoded: 0,
        missing: Vec::new(),
    };
    let first_person_max = std::env::var("BO2MP_LEGACY_TEXTURE")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(FIRST_PERSON_MAX);

    // The guns: every multiplayer weapon's first view and world model, and
    // its rocket (drawn in the launcher).
    let guns: Vec<&LegacyWeaponView> = read.weapons.iter().filter(|w| w.name.to_ascii_lowercase().ends_with("_mp")).collect();
    let mut first_person: BTreeSet<String> = BTreeSet::new();
    let mut third_person: BTreeSet<String> = BTreeSet::new();
    for w in &guns {
        first_person.extend(w.gun_model.clone());
        third_person.extend(w.world_model.clone());
        if let Some(rocket) = w.assets.get("rocketModel") {
            first_person.insert(rocket.clone());
        }
        // The knife the melee pose holds (`viewmodel_knife`, WaW's kbar).
        if let Some(knife) = w.assets.get("knifeModel") {
            first_person.insert(knife.clone());
        }
    }
    let models: HashMap<&str, &iw3::zone::XModel> = read
        .zone
        .assets
        .iter()
        .filter_map(|a| match a {
            iw3::zone::Asset::XModel(m) => Some((m.name.as_str(), m)),
            _ => None,
        })
        .collect();
    let mut fpv = FpvMeshBuild::default();
    fpv.set_capture_ns(ns);
    let mut world_guns = WorldWeaponBuild::default();
    world_guns.set_capture_ns(ns);
    let mut unreadable = Vec::new();
    for (set, max, first) in [(&first_person, first_person_max, true), (&third_person, WORLD_MAX, false)] {
        for name in set {
            let Some(model) = models.get(name.as_str()) else {
                unreadable.push(format!("{name} (not in the zone)"));
                continue;
            };
            let Some(xref) = model_ref(model) else {
                unreadable.push(format!("{name} (no surfaces)"));
                continue;
            };
            let linked: HashMap<usize, usize> = model
                .materials
                .iter()
                .flatten()
                .filter_map(|&id| Some((id, b.material(id, max)?)))
                .collect();
            match asset_model::capture_model_skel_t6(&xref, |k| linked.get(&k.index).copied()) {
                Some(skel) if first => fpv.insert_in(ns, skel, Some(&b.catalog)),
                Some(skel) => world_guns.insert_in(ns, skel, Some(&b.catalog)),
                None => unreadable.push(format!("{name} (unreadable)")),
            }
        }
    }
    report.push(format!(
        "{tag} guest guns: models: {} first person, {} third person built; {} not {:?}",
        fpv.len(),
        world_guns.len(),
        unreadable.len(),
        &unreadable[..unreadable.len().min(8)]
    ));
    report.push(format!(
        "{tag} guest guns: materials: {} linked, {} colour maps decoded from {}, {} not found{}",
        b.materials.len(),
        b.decoded,
        b.main.as_ref().map_or_else(|| "-".to_owned(), |m| m.display().to_string()),
        b.missing.len(),
        b.missing.first().map_or_else(String::new, |m| format!(" (first: {m})"))
    ));

    // Animations every gun names (their names are not case-sensitive).
    let wanted: BTreeSet<String> = guns.iter().flat_map(|w| w.xanims.iter()).filter(|n| !n.is_empty()).map(|n| n.to_ascii_lowercase()).collect();
    let mut xanims = XAnimBuild::default();
    xanims.set_capture_ns(ns);
    let mut anim_n = 0usize;
    for a in &read.anims {
        if wanted.contains(&a.name.to_ascii_lowercase()) {
            anim_n += usize::from(xanims.insert_t6_as(ns, a));
        }
    }
    report.push(format!(
        "{tag} guest guns: animations: {anim_n} of {} named kept ({} gaps)",
        wanted.len(),
        xanims.capture_gaps
    ));

    // The weapon catalog.
    let mut catalog = WeaponCatalog::default();
    catalog.set_capture_ns(ns);
    for w in &guns {
        catalog.capture_legacy(w);
    }
    let captured = catalog.len();
    let mut weapons = catalog.into_build();
    weapons.stamp_namespace(ns);
    weapons.resolve_sz_xanim_edges(&xanims);
    weapons.resolve_fpv_mesh_edges(&fpv);
    weapons.resolve_world_model_edges(&world_guns);
    // bo2mp: each sniper's 2D scope (`overlayMaterial`): its material's
    // picture, which the HUD reads from the game's IWDs by name. (The
    // materials never join the match's definitions, so it was never named
    // and no scope was drawn.) A 2D material's one texture is its picture.
    let overlays = {
        let zone: &iw3::zone::Zone = &read.zone;
        let names = common_material_names(&[zone]);
        weapons.resolve_overlay_images(|material| {
            let key = material.trim_start_matches(',');
            let &(_, i) = names.get(key)?;
            let m = zone.material(i)?;
            let t = m
                .textures
                .iter()
                .find(|t| t.semantic == iw3::zone::TextureSemantic::Color && t.image.is_some())
                .or_else(|| m.textures.iter().find(|t| t.image.is_some()))?;
            let image = zone.image(t.image?)?;
            (!image.name.is_empty()).then(|| image.name.clone())
        })
    };
    report.push(format!("{tag} guest guns: {overlays} scope overlay pictures named"));
    report.push(format!(
        "{tag} guest guns: {captured} weapons captured, {} in the catalog [{:.1} s]",
        weapons.len(),
        started.elapsed().as_secs_f64()
    ));
    let Builder { catalog: material_seed, .. } = b;
    LegacyGuests { weapons, fpv, world_guns, xanims, material_seed, report }
}

/// CoD4's packed unit vector (biased bytes, scaled by the fourth) as Black
/// Ops II's (signed 10:10:10 over 511).
fn iw3_unit_vec_to_t6(packed: u32) -> u32 {
    let n = asset_model::normalize_or_up(asset_model::unpack_unit_vec(packed));
    let mut out = 0u32;
    for (k, v) in n.into_iter().enumerate() {
        let s = (v.clamp(-1.0, 1.0) * 511.0).round() as i32;
        out |= ((s as u32) & 0x3ff) << (10 * k);
    }
    out
}

/// One CoD4 packed vertex as Black Ops II's: the same position and binormal
/// sign; colour R,G,B,A; u in the low half; unit vectors repacked.
fn iw3_vertex_to_t6(v: &iw3::zone::PackedVertex) -> [u8; 32] {
    let mut out = [0u8; 32];
    for k in 0..3 {
        out[k * 4..k * 4 + 4].copy_from_slice(&v.xyz[k].to_le_bytes());
    }
    out[12..16].copy_from_slice(&v.binormal_sign.to_le_bytes());
    let [b, g, r, a] = v.color.to_le_bytes();
    out[16..20].copy_from_slice(&[r, g, b, a]);
    out[20..24].copy_from_slice(&v.tex_coord.rotate_left(16).to_le_bytes());
    out[24..28].copy_from_slice(&iw3_unit_vec_to_t6(v.normal).to_le_bytes());
    out[28..32].copy_from_slice(&iw3_unit_vec_to_t6(v.tangent).to_le_bytes());
    out
}

/// A model's first LOD in Black Ops II's captured form (materials keyed by
/// the zone's asset id).
fn model_ref(m: &iw3::zone::XModel) -> Option<asset_t6::XModelRef> {
    let (first, count) = match m.lods.first() {
        Some(lod) => (usize::from(lod.surf_index), usize::from(lod.num_surfs)),
        None => (0, m.surfs.len()),
    };
    let surfs = m.surfs.get(first..first + count)?;
    if surfs.is_empty() {
        return None;
    }
    let key = |id: usize| asset_t6::AssetKey { ty: fastfile_t6::AssetType::Material, index: id };
    let lod0 = surfs
        .iter()
        .enumerate()
        .map(|(i, s)| asset_t6::XSurfaceRef {
            verts: s.verts.iter().flat_map(iw3_vertex_to_t6).collect(),
            vert_count: s.vert_count,
            indices: s.tris.iter().flatten().copied().collect(),
            vert_lists: s.vert_lists.iter().map(|l| (l.bone_offset, l.vert_count, l.tri_offset, l.tri_count)).collect(),
            flags: if s.deformed { 0x80 } else { 0 },
            material: m.materials.get(first + i).copied().flatten().map(key),
            blend_counts: s.blend_counts,
            blend: s.verts_blend.clone(),
            part_bits: [0; 6],
        })
        .collect();
    let bones = usize::from(m.num_bones);
    Some(asset_t6::XModelRef {
                lighting_origin_offset: [0.0; 3],
                lighting_origin_range: 0.0,
        name: m.name.clone(),
        numsurfs: count.min(255) as u8,
        materials: m.materials.iter().map(|k| k.map(key)).collect(),
        num_bones: m.num_bones,
        num_lods: m.lods.len().max(1) as u16,
        mins: m.mins,
        maxs: m.maxs,
        radius: m.radius,
        lod0,
        bone_names: m.bone_names.clone(),
        num_root_bones: m.num_root_bones,
        parent_list: m.parent_list.clone(),
        quats: m.quats.clone(),
        trans: m.trans.clone(),
        base_mat: m.base_mat.iter().map(|b| (b.quat, b.trans)).collect(),
        part_classification: vec![0; bones],
        bone_info: vec![None; bones],
        lods: m.lods.iter().map(|l| (l.dist, l.num_surfs, l.surf_index)).collect(),
        contents: m.contents as u32,
        coll_lod: -1,
        coll_surfs: Vec::new(),
        phys_preset: None,
    })
}

/// A model material's draw in Black Ops II's pass: by its technique set's
/// name (`mc_l_sm_r0c0s0`, ...) and the state of a lit technique, drawn
/// double-sided, none of BO2's own special kinds.
fn legacy_model_draw(technique_set: &str, sort_key: u8, state: Option<[u32; 2]>) -> T6Draw {
    let mut draw = T6Draw::from_technique_set(technique_set, sort_key, state);
    draw.cull = T6Cull::None;
    draw.water = false;
    draw.flow = false;
    draw.tile = false;
    draw.flag = false;
    draw.tile_blend = false;
    draw.foliage = false;
    draw.objective = false;
    draw.layered = false;
    draw.layers = [0, 0];
    if draw.blend == T6Blend::Sky {
        draw.blend = T6Blend::Opaque;
    }
    // The reflex and holo sights' reticle: a glowing dot, added on unlit.
    if technique_set.split('_').any(|t| t.contains("reflex") || t.contains("holo")) && draw.blend != T6Blend::Opaque {
        draw.unlit = true;
        draw.blend = T6Blend::Add;
        draw.alpha_test = false;
    }
    draw
}

/// Halves an RGBA picture until it fits `max` (a box filter).
fn shrink(mut w: u32, mut h: u32, mut rgba: Vec<u8>, max: u32) -> (u32, u32, Vec<u8>) {
    while (w > max || h > max) && w > 1 && h > 1 {
        let (nw, nh) = (w / 2, h / 2);
        let mut out = vec![0u8; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4 {
                    let at = |xx: u32, yy: u32| u32::from(rgba[((yy * w + xx) * 4 + c) as usize]);
                    let sum = at(2 * x, 2 * y) + at(2 * x + 1, 2 * y) + at(2 * x, 2 * y + 1) + at(2 * x + 1, 2 * y + 1);
                    out[((y * nw + x) * 4 + c) as usize] = (sum / 4) as u8;
                }
            }
        }
        (w, h, rgba) = (nw, nh, out);
    }
    (w, h, rgba)
}

struct Builder<'a> {
    zone: &'a iw3::zone::Zone,
    /// bo2mp: the game's `common_mp`, for a material the zone only names
    /// (a reference: no technique set of its own; its name by name there).
    commons: Vec<&'a iw3::zone::Zone>,
    common_names: Option<HashMap<String, (usize, usize)>>,
    tag: &'static str,
    ns: AssetNamespace,
    main: Option<PathBuf>,
    catalog: MaterialCatalog,
    owner: ZoneOwner,
    /// Zone material id -> catalog material index.
    materials: HashMap<usize, usize>,
    /// (image name, size cap) -> catalog image slot.
    images: HashMap<(String, u32), Option<usize>>,
    decoded: usize,
    missing: Vec<String>,
}

impl<'a> Builder<'a> {
    /// A colour map from the game's IWDs (flat grey where it is not found).
    fn image(&mut self, name: &str, max: u32) -> Option<usize> {
        let key = (name.to_owned(), max);
        if let Some(slot) = self.images.get(&key) {
            return *slot;
        }
        let pixels = self
            .main
            .as_deref()
            .and_then(|main| asset_material::decode_ui_image_from_main(main, name).ok().flatten());
        let (w, h, rgba) = match pixels {
            Some((w, h, rgba)) if rgba.len() == (w * h * 4) as usize => {
                self.decoded += 1;
                shrink(w, h, rgba, max)
            }
            _ => {
                if self.missing.len() < 64 {
                    self.missing.push(name.to_owned());
                }
                (1, 1, vec![128, 128, 128, 255])
            }
        };
        let image = Image::new(
            Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            TextureDimension::D2,
            rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let slot = Some(self.catalog.link_image(AuthoredImage {
            namespace: self.ns,
            name: AssetRef::Real(format!("{}:{name}#{max}", self.tag)),
            map_type: 0,
            semantic: TS_COLOR_MAP,
            category: 0,
            use_srgb_reads: true,
            width: w as u16,
            height: h as u16,
            depth: 1,
            level_count: 1,
            format: 0,
            payload: Arc::new(Vec::new()),
            decoded: Some(Arc::new(image)),
            common_owned: false,
            decoded_variant: None,
            decoded_by: None,
            pending_decode: None,
        }));
        self.images.insert(key, slot);
        slot
    }

    /// A zone material, or for a reference (no technique set: the zone
    /// only names it) the game's `common_mp` material of that name.
    fn resolve(&mut self, id: usize) -> Option<(&'a iw3::zone::Zone, &'a iw3::zone::Material)> {
        let zone: &'a iw3::zone::Zone = self.zone;
        let m = zone.material(id)?;
        if m.technique_set.is_some() && !m.name.starts_with(',') {
            return Some((zone, m));
        }
        if self.commons.is_empty() {
            return Some((zone, m));
        }
        let commons = self.commons.clone();
        let names = self.common_names.get_or_insert_with(|| common_material_names(&commons));
        match names.get(m.name.trim_start_matches(',')).and_then(|&(z, i)| Some((commons[z], commons[z].material(i)?))) {
            Some(found) => Some(found),
            None => Some((zone, m)),
        }
    }

    /// One model material into the catalog, with its colour map.
    fn material(&mut self, id: usize, max: u32) -> Option<usize> {
        if let Some(&i) = self.materials.get(&id) {
            return Some(i);
        }
        let (zone, m) = self.resolve(id)?;
        let technique_set = m.technique_set.and_then(|t| zone.technique_set(t)).map_or("", |t| t.name.as_str());
        let state = TECHNIQUES.iter().find_map(|&t| m.state_bits_for(t));
        let draw = legacy_model_draw(technique_set, m.sort_key, state);
        let mut textures = Vec::new();
        let colour = m
            .textures
            .iter()
            .find(|t| t.semantic == iw3::zone::TextureSemantic::Color && t.image.is_some());
        if let Some(t) = colour
            && let Some(image) = t.image.and_then(|i| zone.image(i))
        {
            let slot = self.image(&image.name, max);
            textures.push(MaterialTextureBinding {
                name_hash: t.name_hash,
                name_start: 0,
                name_end: 0,
                sampler_state: t.sampler_state,
                semantic: TS_COLOR_MAP,
                image: slot,
            });
        }
        let index = self.catalog.link_material(AuthoredMaterial {
            name: AssetRef::Real(format!("{}:{}", self.tag, m.name)),
            namespace: self.ns,
            technique_set: AssetRef::default(),
            technique_set_edge: AssetEdge::Absent,
            draw_surf: 0,
            sort_key: m.sort_key,
            info_game_flags: 0,
            texture_atlas: None,
            surface_type_bits: None,
            t5_layered_surface_types: None,
            state_flags: 0,
            camera_region: 0,
            state_bits: Vec::new(),
            state_bits_entry: None,
            t5_state_bits_entry: None,
            iw5_state_bits_entry: None,
            technique_table: None,
            route: None,
            textures,
            constants: Vec::new(),
            zone: self.owner,
            t6_draw: Some(draw),
        });
        self.materials.insert(id, index);
        Some(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_vectors_repack() {
        // +x as CoD4 packs it (255, 127, 127, scale byte 0x3f): 511 in BO2's.
        let packed = u32::from_le_bytes([255, 127, 127, 63]);
        assert_eq!(iw3_unit_vec_to_t6(packed) & 0x3ff, 511);
    }

    #[test]
    fn shrinks_to_fit() {
        let (w, h, px) = shrink(4, 4, vec![200; 64], 2);
        assert_eq!((w, h, px.len()), (2, 2, 16));
        assert!(px.iter().all(|&v| v == 200));
    }
}


/// bo2mp: a CoD4 or World at War map's placed models (the GfxWorld's static
/// models: statues, debris, furniture, flags), their materials linked into
/// `catalog` (a reference resolved from `common`), each lit by the map's
/// light grid at its middle.
pub(crate) struct LegacyProps {
    pub catalog: MaterialCatalog,
    pub meshes: Vec<asset_world::ModelMesh>,
    pub placements: Vec<Option<asset_world::StaticModelPlacement>>,
    pub report: String,
}

/// The biggest side of a prop's picture.
const PROP_MAX: u32 = 1024;

pub(crate) fn legacy_static_models(
    tag: &'static str,
    ns: AssetNamespace,
    path: &Path,
    zone: &iw3::zone::Zone,
    commons: &[&iw3::zone::Zone],
    world: &iw3::zone::GfxWorld,
    catalog: MaterialCatalog,
) -> LegacyProps {
    use bevy::prelude::{Mat3, Quat, Transform, Vec3};
    let main = asset_transport::game_root_for_zone(path).ok().map(|root| root.join("main"));
    let mut b = Builder {
        zone,
        commons: commons.to_vec(),
        common_names: None,
        tag,
        ns,
        main,
        catalog,
        owner: ZoneOwner::from_zone_path(path),
        materials: HashMap::new(),
        images: HashMap::new(),
        decoded: 0,
        missing: Vec::new(),
    };
    // bo2mp: the light grid as the moving things read it
    // (`world_iw3::iw3_owned_light_grid`): the lookup's mean colour and the
    // sun's visibility at the model's middle.
    let grid = asset_world::world_iw3::iw3_owned_light_grid(&world.light_grid);
    let sun_primary = grid.as_ref().map_or(0, |g| g.sun_primary_light_index.min(255) as u8);
    // (A middle the lookup cannot answer, inside a wall or under the
    // floor: a little higher, then the map's mean light without the sun.
    // A placement without light is not drawn.)
    let mean: Option<[f32; 3]> = (!world.light_grid.colors.is_empty()).then(|| {
        let mut sum = [0.0f32; 3];
        for c in &world.light_grid.colors {
            for d in &c.0 {
                for k in 0..3 {
                    sum[k] += f32::from(d[k]) / 255.0;
                }
            }
        }
        sum.map(|v| v / (world.light_grid.colors.len() * 56) as f32)
    });
    let light_at = |at: [f32; 3]| {
        let g = grid.as_ref()?;
        [0.0, 16.0, 48.0]
            .iter()
            .find_map(|dz| asset_world::world_iw3::iw3_grid_light(g, [at[0], at[1], at[2] + dz]))
            .or_else(|| mean.map(|m| (m, 0.0)))
    };
    let mut meshes = Vec::new();
    let mut shared: HashMap<usize, Option<(usize, [f32; 3])>> = HashMap::new();
    let mut placements = vec![None; world.static_models.len()];
    let (mut lit, mut dark, mut unreadable) = (0usize, 0usize, Vec::new());
    for (i, sm) in world.static_models.iter().enumerate() {
        let Some(id) = sm.model else { continue };
        let entry = *shared.entry(id).or_insert_with(|| {
            let model = zone.xmodel(id)?;
            let Some(xref) = model_ref(model) else {
                if unreadable.len() < 8 {
                    unreadable.push(model.name.clone());
                }
                return None;
            };
            let linked: HashMap<usize, usize> = model
                .materials
                .iter()
                .flatten()
                .filter_map(|&mid| Some((mid, b.material(mid, PROP_MAX)?)))
                .collect();
            let mesh = asset_world::build_t6_model_mesh(&xref, |k| linked.get(&k.index).copied())?;
            meshes.push(mesh);
            let mid: [f32; 3] = std::array::from_fn(|k| (xref.mins[k] + xref.maxs[k]) * 0.5);
            Some((meshes.len() - 1, mid))
        });
        let Some((mesh, mid)) = entry else { continue };
        let at: [f32; 3] = std::array::from_fn(|k| {
            sm.origin[k] + (mid[0] * sm.axis[0][k] + mid[1] * sm.axis[1][k] + mid[2] * sm.axis[2][k]) * sm.scale
        });
        // As CoD4's / World at War's lit model shaders (`lp_*`, read from
        // common_mp): twice the grid's colour, in gamma space (light =
        // (2 * colour)^2, packed sqrt(light / 32)), and the sun by its
        // visibility there (`t6_primary`; the fallback pass adds it in
        // gamma space on these maps).
        let (packed_lighting, t6_primary) = match light_at(at) {
            Some((c, sun)) => {
                lit += 1;
                let pack = |v: f32| (2.0 * v / 32f32.sqrt() * 255.0).round().clamp(0.0, 254.0) as u8;
                (Some([pack(c[0]), pack(c[1]), pack(c[2]), 0]), [sun_primary, (sun * 255.0).round() as u8])
            }
            None => {
                dark += 1;
                (None, [0, 0])
            }
        };
        let basis = Mat3::from_cols(Vec3::from_array(sm.axis[0]), Vec3::from_array(sm.axis[1]), Vec3::from_array(sm.axis[2]));
        placements[i] = Some(asset_world::StaticModelPlacement {
            mesh,
            transform: Transform {
                translation: Vec3::from_array(sm.origin),
                rotation: Quat::from_mat3(&basis),
                scale: Vec3::splat(sm.scale),
            },
            origin: sm.origin,
            axis: sm.axis,
            scale: sm.scale,
            cull_dist: sm.cull_dist.clamp(0.0, f32::from(u16::MAX)) as u16,
            reflection_probe_index: 0,
            primary_light_index: 0,
            flags: sm.flags,
            packed_lighting,
            t6_primary,
        });
    }
    let report = format!(
        "{tag} props: {} placed of {}, {} models; {lit} lit by the light grid, {dark} outside it; {} pictures decoded, {} not found{}; unreadable {:?}",
        placements.iter().filter(|p| p.is_some()).count(),
        world.static_models.len(),
        meshes.len(),
        b.decoded,
        b.missing.len(),
        if b.missing.is_empty() { String::new() } else { format!(" (first: {})", b.missing.iter().take(4).cloned().collect::<Vec<_>>().join(", ")) },
        unreadable
    );
    let Builder { catalog, .. } = b;
    LegacyProps { catalog, meshes, placements, report }
}

/// bo2mp: the full materials of the zones a map leans on (the game's
/// localized_common_mp first: the flags; then common_mp), by name: (zone,
/// material id). The first zone to define a name keeps it.
pub(crate) fn common_material_names(commons: &[&iw3::zone::Zone]) -> HashMap<String, (usize, usize)> {
    let mut out = HashMap::new();
    for (z, c) in commons.iter().enumerate() {
        for (i, a) in c.assets.iter().enumerate() {
            if let iw3::zone::Asset::Material(m) = a
                && m.technique_set.is_some()
                && !m.name.starts_with(',')
            {
                out.entry(m.name.trim_start_matches(',').to_owned()).or_insert((z, i));
            }
        }
    }
    out
}
