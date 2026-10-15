//! A multiplayer or zombies map (`mp_*.ff`, `cp_*.ff`) read whole: the map
//! zone, its materials (`techsets_<map>.ff`), its models' surfaces (the
//! zone's shared streams in `assetfile<N>.pak`) and its images' pixels
//! (`imagefile<N>.pak`), in plain terms a converter can use: the world's
//! vertices, indices and surfaces, the static models and their meshes, the
//! materials with their image names, the spawn points and the entities.
//!
//! IW7 keeps its entity string's keys as canonical string numbers; the ones
//! a game's scripts read are named here (`ENTITY_KEYS`), others are kept as
//! `_k<number>`. Spawn points are not in the entity string: they are the
//! zone's spawn list (`MapEnts::spawnList`), added to the entities.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::asset_type::AssetType;
use crate::container::{self, StreamFile};
use crate::texture::{self, ImageLevels};
use crate::walk::{self, AssetId, Capture, ImageRec};

/// The entity keys whose canonical string numbers are known (by their
/// values on the shipped maps).
pub const ENTITY_KEYS: &[(u32, &str)] = &[
    (65, "angles"),
    (72, "animscript"),
    (157, "classname"),
    (337, "spawnflags"),
    (371, "radius"),
    (497, "model"),
    (543, "origin"),
    (599, "height"),
    (636, "script_linkname"),
    (638, "script_noteworthy"),
    (719, "suncolor"),
    (720, "sundirection"),
    (721, "sunlight"),
    // (820 holds the editor's link names, `auto601`, `pf7_auto6`; 822 the
    // names scripts look entities up by, `heli_attack_area`, `door_buy`.)
    (820, "target"),
    (822, "targetname"),
    (61049, "script_parameters"),
    (60863, "script_gameobjectname"),
    (60921, "script_label"),
    (60929, "script_linkto"),
];

/// One world vertex (`GfxWorldVertex`, 44 bytes), unpacked.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldVertex {
    pub xyz: [f32; 3],
    pub binormal_sign: f32,
    /// D3DCOLOR (B, G, R, A bytes).
    pub color: u32,
    pub uv: [f32; 2],
    pub lmap_uv: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
}

/// IW7's packed unit vector: unsigned 10:10:10 over 1023.
pub fn unpack_unit(packed: u32) -> [f32; 3] {
    std::array::from_fn(|k| ((packed >> (10 * k)) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0)
}

/// One world surface: its triangles index the world's index buffer from
/// `base_index`, relative to `first_vertex` (already offset into the
/// concatenated vertices of every transient zone).
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldSurface {
    pub first_vertex: u32,
    pub vertex_count: u16,
    pub tri_count: u16,
    pub base_index: u32,
    pub material: Option<usize>,
    pub lightmap_index: u8,
    pub flags: u8,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    /// The brush model (`*<n>`) the surface is part of; 0 the world's own.
    pub brush_model: u32,
    /// A brush model's surface not shown as the map starts (see
    /// `Iw7Map::place_brush_models`): not drawn and not collided with.
    pub hidden_brush: bool,
}

/// A material: its technique set's name, sort key and textures by sampler
/// (name hash, sampler state, semantic, image name).
#[derive(Clone, Debug, Default)]
pub struct MapMaterial {
    pub name: String,
    pub technique_set: String,
    pub sort_key: u8,
    pub material_type: u8,
    pub textures: Vec<(u32, u8, u8, String)>,
    /// (name, value) of the literal constants.
    pub constants: Vec<(String, [f32; 4])>,
}

impl MapMaterial {
    /// The image a sampler (`colorMap`, `normalMap`, ...) reads.
    pub fn texture(&self, sampler: &str) -> Option<&str> {
        let h = texture::name_hash(sampler);
        self.textures.iter().find(|t| t.0 == h).map(|t| t.3.as_str())
    }
}

/// One model surface: `GfxPackedVertex`es (32 bytes each) and triangles.
#[derive(Clone, Debug, Default)]
pub struct ModelSurface {
    pub verts: Vec<[u8; 32]>,
    pub tris: Vec<[u16; 3]>,
    pub material: Option<usize>,
}

/// A static model's mesh (its first LOD whose surfaces were found).
#[derive(Clone, Debug, Default)]
pub struct MapModel {
    pub name: String,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub radius: f32,
    pub surfaces: Vec<ModelSurface>,
    /// The coarsest LOD found (for collision); empty when it is the first.
    pub coarse: Vec<ModelSurface>,
}

/// A placed static model.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticModel {
    pub model: usize,
    pub origin: [f32; 3],
    /// Rows: the model's forward, left and up axes in the world.
    pub axis: [[f32; 3]; 3],
    pub scale: f32,
    pub cull_dist: f32,
    /// Where the model takes its light from (`GfxStaticModelInst`).
    pub lighting_origin: [f32; 3],
    /// Its index among the world's placed models.
    pub index: u32,
}

/// A spawn point from the zone's spawn list.
#[derive(Clone, Debug, Default)]
pub struct SpawnPoint {
    pub classname: String,
    pub targetname: String,
    pub target: String,
    pub script_noteworthy: String,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
}

/// A scriptable instance: its definition's and model's names, where it
/// stands and its script name.
#[derive(Clone, Debug, Default)]
pub struct ScriptableInstance {
    pub def: String,
    pub model: String,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub targetname: String,
}

/// A brush sub-model (`*<n>`: a door, a debris pile): its world surfaces
/// (`GfxBrushModel`) and its bounds.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrushModel {
    pub first_surface: u32,
    pub surface_count: u32,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
}

/// An AI path node (`pathnode_t`) and its links (node, distance,
/// negotiation link).
#[derive(Clone, Debug, Default)]
pub struct PathNode {
    pub kind: u16,
    pub spawnflags: u16,
    pub targetname: String,
    pub target: String,
    pub script_noteworthy: String,
    pub script_linkname: String,
    pub animscript: String,
    pub origin: [f32; 3],
    pub yaw: f32,
    pub links: Vec<(u16, f32, bool)>,
}

/// A primary light (`ComPrimaryLight`): the sun is type 1.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrimaryLight {
    pub kind: u8,
    pub color: [f32; 3],
    pub dir: [f32; 3],
    pub origin: [f32; 3],
    pub radius: f32,
}

/// Where a map's files are.
#[derive(Clone, Debug)]
pub struct MapSources {
    pub zone: PathBuf,
    /// `techsets_<map>.ff`: the map's materials.
    pub techsets: Option<PathBuf>,
    /// The folder of `assetfile<N>.pak` and `imagefile<N>.pak`.
    pub pak_dir: PathBuf,
    /// Instead of the `assetfile` paks: the zone's shared ranges copied end
    /// to end into one file (as `iw7_shared` prints how to).
    pub shared_ranges: Option<PathBuf>,
    /// Zones whose images (and materials) a map may name but not define
    /// (common zones), searched after the map's own.
    pub extra_zones: Vec<PathBuf>,
    /// Zones whose models a map places but does not define (mp_hawkwar's
    /// cluster grenades and debris are `common_core_mp`'s), each with its
    /// material zone; read only when a placed model is missing.
    pub model_zones: Vec<(PathBuf, Option<PathBuf>)>,
    /// The common zones' pictures and materials (`crate::shared`), for
    /// those the map names but does not hold; read once by the caller.
    pub shared: Option<std::sync::Arc<crate::shared::SharedZones>>,
}

impl MapSources {
    /// A map zone in the game's folder: `techsets_<map>.ff` and the paks
    /// beside it.
    pub fn beside(zone: &Path) -> Self {
        let dir = zone.parent().map(Path::to_path_buf).unwrap_or_default();
        let stem = zone.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let techsets = [format!("techsets_{stem}.ff"), format!("techsets_{}.ff", stem.trim_start_matches("iw7_"))]
            .into_iter()
            .chain(stem.strip_prefix("iw7_").map(|s| format!("iw7_techsets_{s}.ff")))
            .map(|n| dir.join(n))
            .find(|p| p.is_file());
        let model_zones = ["common_core_mp"]
            .into_iter()
            .map(|z| (dir.join(format!("{z}.ff")), Some(dir.join(format!("techsets_{z}.ff"))).filter(|p| p.is_file())))
            .filter(|(p, _)| p.is_file())
            .collect();
        // A Zombies map's own patch zone (`patch_cp_zmb.ff`: 1449 pictures
        // on cp_zmb, among them signs and posters the map names): read for
        // what the map zone lacks.
        let extra_zones = if stem.trim_start_matches("iw7_").starts_with("cp_") {
            [dir.join(format!("patch_{}.ff", stem.trim_start_matches("iw7_")))].into_iter().filter(|p| p.is_file()).collect()
        } else {
            Vec::new()
        };
        Self { zone: zone.to_owned(), techsets, pak_dir: dir, shared_ranges: None, extra_zones, model_zones, shared: None }
    }

    /// With the common zones beside the map read for the pictures and
    /// materials it names but does not hold (`crate::shared`).
    pub fn with_common(mut self) -> Self {
        let stem = self.zone.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let zones = crate::shared::common_zones_for(&self.pak_dir, &stem);
        self.shared = Some(std::sync::Arc::new(crate::shared::SharedZones::read(&zones)));
        self
    }
}

/// A map, read.
#[derive(Default)]
pub struct Iw7Map {
    pub name: String,
    pub vertices: Vec<WorldVertex>,
    pub indices: Vec<u16>,
    pub surfaces: Vec<WorldSurface>,
    /// Per sky: its surfaces.
    pub sky_surfaces: Vec<u32>,
    pub materials: Vec<MapMaterial>,
    pub models: Vec<MapModel>,
    pub static_models: Vec<StaticModel>,
    pub spawns: Vec<SpawnPoint>,
    /// The entities as a classic entity string (named keys, spawn points
    /// added).
    pub entities: String,
    pub lights: Vec<PrimaryLight>,
    /// The `GfxLightMap`s: their primary (BC4 sun visibility), secondary
    /// (BC6H, two pages stacked) and second-unorm (R8G8 dominant light
    /// directions) image names.
    pub light_maps: Vec<[String; 3]>,
    /// The map's vision (`vision/<map>.vision`, else the first vision in
    /// the zone): its name and text.
    pub vision: Option<(String, String)>,
    /// The reflection probes' cube array image (`*reflection_probe_array`)
    /// and the probes' origins.
    pub probe_image: Option<String>,
    pub probe_origins: Vec<[f32; 3]>,
    /// The map's scriptable instances (`MapEnts::scriptableMapEnts`):
    /// destructible and scripted props (Zombies maps' buyable blockers
    /// among them), not among the placed models.
    pub scriptables: Vec<ScriptableInstance>,
    /// The server triggers' hulls by `?<n>` (a volume's or trigger's
    /// `model`): each hull's midpoint and half size, about the entity's
    /// origin.
    pub trigger_hulls: Vec<Vec<([f32; 3], [f32; 3])>>,
    /// The brush sub-models by `*<n>` (0 is the world itself).
    pub brush_models: Vec<BrushModel>,
    /// The AI path nodes (`PathData`), in node order.
    pub path_nodes: Vec<PathNode>,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    /// What was read and what was missing, line by line.
    pub report: Vec<String>,
    /// Materials taken from the common zones (`MapSources::shared`).
    pub shared_materials: usize,
    images: HashMap<String, ImageSource>,
    pak_dir: PathBuf,
    shared: Option<std::sync::Arc<crate::shared::SharedZones>>,
}

/// Where an image's pixels are.
#[derive(Clone)]
struct ImageSource {
    rec: ImageRec,
    /// Inline pixels (all levels).
    inline: Option<Vec<u8>>,
}

fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(b[o..o + 2].try_into().unwrap())
}
fn v3(b: &[u8], o: usize) -> [f32; 3] {
    [f32_at(b, o), f32_at(b, o + 4), f32_at(b, o + 8)]
}

fn bare(name: &str) -> &str {
    name.trim_start_matches(',')
}

/// A zone walked, with what of it is needed after the walk.
struct Walked {
    zone: crate::Zone,
    cap: Capture,
}

fn walk_file(path: &Path) -> Result<Walked, String> {
    let zone = crate::Zone::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (w, stop) = walk::walk_fitted(&zone);
    let cap = w.cap;
    if let Some(stop) = stop {
        return Err(format!("{}: walk stopped at asset {} ({:?}): {}", path.display(), stop.asset_index, stop.asset_type, stop.error));
    }
    Ok(Walked { zone, cap })
}

/// The zone's shared ranges, read from the `assetfile` paks (or the copy).
fn shared_data(src: &MapSources, header: &container::FileHeader, copy: Option<&Path>) -> Result<crate::pak::SharedVirtual, String> {
    let raw = match copy {
        Some(p) => std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?,
        None => {
            let mut raw = Vec::new();
            for f in &header.shared_files {
                let pak = src.pak_dir.join(format!("assetfile{}.pak", f.file_index));
                raw.extend(crate::pak::read_range(&pak, f).map_err(|e| format!("{}: {e}", pak.display()))?);
            }
            raw
        }
    };
    crate::pak::SharedVirtual::from_concatenated(&raw, &header.shared_files).map_err(|e| format!("shared ranges: {e}"))
}

impl Iw7Map {
    /// Reads a map. Errors only when the map zone itself cannot be read;
    /// a missing material zone or shared stream leaves gaps (see `report`).
    pub fn load(src: &MapSources) -> Result<Self, String> {
        let map = walk_file(&src.zone)?;
        let mut out = Iw7Map {
            name: src.zone.file_stem().map(|s| s.to_string_lossy().trim_start_matches("iw7_").to_owned()).unwrap_or_default(),
            pak_dir: src.pak_dir.clone(),
            shared: src.shared.clone(),
            ..Default::default()
        };
        // Materials: the map's material zone, then the map and extra zones.
        let mut zones: Vec<Walked> = Vec::new();
        match &src.techsets {
            Some(p) => match walk_file(p) {
                Ok(w) => zones.push(w),
                Err(e) => out.report.push(format!("material zone: {e}")),
            },
            None => out.report.push("material zone: no techsets_<map>.ff beside the map".to_owned()),
        }
        for p in &src.extra_zones {
            match walk_file(p) {
                Ok(w) => zones.push(w),
                Err(e) => out.report.push(format!("extra zone: {e}")),
            }
        }
        // Placed models the map zone names but does not hold (an XModel
        // with no LOD surfaces): read the model zones for them, their
        // materials and pictures with the map's.
        let has_mesh = |cap: &Capture, name: &str| {
            cap.xmodels.iter().any(|x| bare(&x.name) == name && x.lods.iter().any(|l| l.numsurfs > 0 && l.surfs.is_some()))
        };
        let wanted: std::collections::BTreeSet<String> = map
            .cap
            .gfx_worlds
            .first()
            .map(|g| g.smodel_models.iter().flatten().map(|&id| bare(map.cap.name_of(id)).to_owned()).collect())
            .unwrap_or_default();
        let missing: Vec<&String> = wanted.iter().filter(|n| !has_mesh(&map.cap, n)).collect();
        let mut model_zones: Vec<Walked> = Vec::new();
        if !missing.is_empty() {
            for (z, techsets) in &src.model_zones {
                match walk_file(z) {
                    Ok(w) => {
                        if !missing.iter().any(|n| has_mesh(&w.cap, n)) {
                            continue;
                        }
                        if let Some(t) = techsets {
                            match walk_file(t) {
                                Ok(t) => zones.push(t),
                                Err(e) => out.report.push(format!("model zone materials: {e}")),
                            }
                        }
                        model_zones.push(w);
                    }
                    Err(e) => out.report.push(format!("model zone: {e}")),
                }
            }
        }

        // Images by name: the map's first.
        for w in std::iter::once(&map).chain(zones.iter()).chain(model_zones.iter()) {
            for im in &w.cap.images {
                if im.name.starts_with(',') || out.images.contains_key(&im.name) {
                    continue;
                }
                let inline = im.pixels.map(|s| s.get(&w.zone.image).to_vec());
                if inline.is_none() && !im.streamed {
                    continue;
                }
                out.images.insert(im.name.clone(), ImageSource { rec: im.clone(), inline });
            }
        }
        let mut material_defs: HashMap<String, (usize, usize)> = HashMap::new();
        for (z, w) in std::iter::once(&map).chain(zones.iter()).chain(model_zones.iter()).enumerate() {
            for (i, m) in w.cap.materials.iter().enumerate() {
                if !m.name.starts_with(',') {
                    material_defs.entry(m.name.clone()).or_insert((z, i));
                }
            }
        }
        let all: Vec<&Walked> = std::iter::once(&map).chain(zones.iter()).chain(model_zones.iter()).collect();
        let mut material_index: HashMap<String, Option<usize>> = HashMap::new();
        let mut missing_materials = Vec::new();
        let mut material_of = |out: &mut Iw7Map, cap: &Capture, id: Option<AssetId>| -> Option<usize> {
            let name = bare(cap.name_of(id?)).to_owned();
            if let Some(i) = material_index.get(&name) {
                return *i;
            }
            let shared = out.shared.clone();
            let from_shared = |out: &mut Iw7Map| -> Option<usize> {
                let m = shared.as_ref()?.material(&name)?;
                out.materials.push(m.clone());
                out.shared_materials += 1;
                Some(out.materials.len() - 1)
            };
            let found = material_defs.get(&name).map(|&(z, i)| {
                let w = all[z];
                let m = &w.cap.materials[i];
                out.materials.push(MapMaterial {
                    name: name.clone(),
                    technique_set: m.techset.map(|t| bare(w.cap.name_of(t)).to_owned()).unwrap_or_default(),
                    sort_key: m.sort_key,
                    material_type: m.material_type,
                    textures: m
                        .textures
                        .iter()
                        .filter_map(|t| Some((t.name_hash, t.sampler_state, t.semantic, bare(w.cap.name_of(t.image?)).to_owned())))
                        .collect(),
                    constants: m.constants.iter().map(|c| (c.name.clone(), c.literal)).collect(),
                });
                out.materials.len() - 1
            });
            let found = found.or_else(|| from_shared(out));
            if found.is_none() && missing_materials.len() < 12 {
                missing_materials.push(name.clone());
            }
            material_index.insert(name, found);
            found
        };

        // The world.
        let image = &map.zone.image;
        let Some(g) = map.cap.gfx_worlds.first() else {
            return Err(format!("{}: no GfxWorld", src.zone.display()));
        };
        let gh = g.header;
        use crate::layout as L;
        // Transient zones: each one's vertices, concatenated by index.
        let mut tz_base: HashMap<u32, u32> = HashMap::new();
        let mut trs: Vec<&walk::GfxWorldTrRec> = g
            .transient_zones
            .iter()
            .flatten()
            .filter_map(|id| (id.ty == AssetType::GfxWorldTransientZone).then(|| map.cap.gfx_world_trs.get(id.index as usize)).flatten())
            .collect();
        trs.sort_by_key(|t| t.index);
        for t in &trs {
            let Some(at) = t.vertices else {
                out.report.push(format!("transient zone {} #{}: vertices not in the zone", t.name, t.index));
                continue;
            };
            tz_base.insert(t.index, out.vertices.len() as u32);
            for i in 0..t.vertex_count as usize {
                let v = &image[at + 44 * i..at + 44 * (i + 1)];
                out.vertices.push(WorldVertex {
                    xyz: v3(v, 0),
                    binormal_sign: if f32_at(v, 12) < 0.0 { -1.0 } else { 1.0 },
                    color: u32_at(v, 16),
                    uv: [f32_at(v, 20), f32_at(v, 24)],
                    lmap_uv: [f32_at(v, 28), f32_at(v, 32)],
                    normal: unpack_unit(u32_at(v, 36)),
                    tangent: unpack_unit(u32_at(v, 40)),
                });
            }
        }
        if let Some(at) = g.indices {
            out.indices = (0..g.index_count as usize).map(|i| u16_at(image, at + 2 * i)).collect();
        }
        let mut no_zone = 0usize;
        if let Some(at) = g.surfaces {
            use L::GfxSurface as S;
            for i in 0..g.surface_count as usize {
                let s = &image[at + S::SIZE * i..at + S::SIZE * (i + 1)];
                let tz = u32::from(s[S::transientZone]);
                let Some(&base) = tz_base.get(&tz) else {
                    no_zone += 1;
                    out.surfaces.push(WorldSurface::default());
                    continue;
                };
                let bounds = g.surface_bounds.map(|b| {
                    let o = b + L::GfxSurfaceBounds::SIZE * i;
                    let (mid, half) = (v3(image, o), v3(image, o + 12));
                    (std::array::from_fn(|k| mid[k] - half[k]), std::array::from_fn(|k| mid[k] + half[k]))
                });
                let material = material_of(&mut out, &map.cap, g.surface_materials.get(i).copied().flatten());
                out.surfaces.push(WorldSurface {
                    first_vertex: base + u32_at(s, S::tris__firstVertex),
                    vertex_count: u16_at(s, S::tris__vertexCount),
                    tri_count: u16_at(s, S::tris__triCount),
                    base_index: u32_at(s, S::tris__baseIndex),
                    material,
                    lightmap_index: s[S::lightmapIndex],
                    flags: s[S::flags],
                    mins: bounds.map_or([0.0; 3], |b| b.0),
                    maxs: bounds.map_or([0.0; 3], |b| b.1),
                    brush_model: 0,
                    hidden_brush: false,
                });
            }
        }
        if no_zone > 0 {
            out.report.push(format!("{no_zone} surfaces in transient zones not read"));
        }
        for s in &g.sky_surfs {
            out.sky_surfaces.extend(s.iter().filter_map(|&k| u32::try_from(k).ok()));
        }
        let mid = v3(image, gh + L::GfxWorld::bounds__midPoint);
        let half = v3(image, gh + L::GfxWorld::bounds__halfSize);
        out.mins = std::array::from_fn(|k| mid[k] - half[k]);
        out.maxs = std::array::from_fn(|k| mid[k] + half[k]);

        // Static models and their meshes (surfaces from the shared streams).
        let shared = match shared_data(src, &map.zone.header, src.shared_ranges.as_deref()) {
            Ok(sv) => {
                let (cap, stops) = walk::walk_shared(&sv);
                if !stops.is_empty() {
                    out.report.push(format!("shared streams: {} ranges did not read to their end", stops.len()));
                }
                Some((sv, cap))
            }
            Err(e) => {
                out.report.push(format!("shared streams: {e} (static models not drawn)"));
                None
            }
        };
        // Where models are: the map zone (its shared streams, else the
        // zone itself: mp_riot's vista mountains, `ml/periph_*`), then the
        // model zones read for what it lacks.
        struct ModelSource<'a> {
            cap: &'a Capture,
            image: &'a [u8],
            shared: Option<(crate::pak::SharedVirtual, Capture)>,
        }
        let mut sources = vec![ModelSource { cap: &map.cap, image: image.as_slice(), shared }];
        for w in &model_zones {
            let shared = match shared_data(src, &w.zone.header, None) {
                Ok(sv) => {
                    let (cap, _) = walk::walk_shared(&sv);
                    Some((sv, cap))
                }
                Err(e) => {
                    out.report.push(format!("model zone shared streams: {e}"));
                    None
                }
            };
            sources.push(ModelSource { cap: &w.cap, image: w.zone.image.as_slice(), shared });
        }
        let surfs_by_name: Vec<HashMap<&str, usize>> = sources
            .iter()
            .map(|s| s.shared.as_ref().map(|(_, c)| c.model_surfs.iter().enumerate().map(|(i, m)| (bare(&m.name), i)).collect()).unwrap_or_default())
            .collect();
        let zone_surfs: Vec<HashMap<&str, usize>> = sources
            .iter()
            .map(|s| {
                s.cap
                    .model_surfs
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| !m.name.starts_with(',') && m.surfs.iter().any(|s| s.verts.is_some()))
                    .map(|(i, m)| (m.name.as_str(), i))
                    .collect()
            })
            .collect();
        let mut model_index: HashMap<String, Option<usize>> = HashMap::new();
        let mut no_mesh = Vec::new();
        let mut from_model_zones = 0usize;
        // A model placed twice exactly in one place (mp_riot has 45, mp_proto
        // 39): drawn once (twice, its faces flickered through each other).
        let mut placed_at: std::collections::HashSet<(usize, [i64; 12])> = std::collections::HashSet::new();
        let mut doubled = 0usize;
        if let Some(at) = g.smodels {
            use L::GfxStaticModelDrawInst as D;
            for i in 0..g.smodel_count as usize {
                let d = &image[at + D::SIZE * i..at + D::SIZE * (i + 1)];
                let Some(id) = g.smodel_models.get(i).copied().flatten() else { continue };
                let name = bare(map.cap.name_of(id)).to_owned();
                let model = match model_index.get(&name) {
                    Some(m) => *m,
                    None => {
                        // The first source whose XModel of that name (a
                        // comma-named one too: mp_geneva's auto-LOD props
                        // are held under their reference name) has a LOD
                        // whose surfaces are found.
                        let mut built = None;
                        for (si, sr) in sources.iter().enumerate() {
                            // Where a LOD's surfaces are: the shared streams,
                            // else the zone.
                            let source = |l: &walk::LodRec| -> Option<(&walk::ModelSurfsRec, &[u8])> {
                                let n = bare(sr.cap.name_of(l.surfs?));
                                if let (Some((sv, scap)), Some(&k)) = (sr.shared.as_ref(), surfs_by_name[si].get(n)) {
                                    return Some((&scap.model_surfs[k], sv.data.as_slice()));
                                }
                                zone_surfs[si].get(n).map(|&k| (&sr.cap.model_surfs[k], sr.image))
                            };
                            let mut candidates: Vec<&walk::XModelRec> = sr.cap.xmodels.iter().filter(|x| bare(&x.name) == name).collect();
                            candidates.sort_by_key(|x| x.name.starts_with(','));
                            let Some((x, found)) = candidates.into_iter().find_map(|x| {
                                let found: Vec<&walk::LodRec> = x.lods.iter().filter(|l| l.numsurfs > 0 && source(l).is_some()).collect();
                                (!found.is_empty()).then_some((x, found))
                            }) else {
                                continue;
                            };
                            // The LODs whose surfaces were found: the first is
                            // drawn, the last is the collision's.
                            let mut read = |out: &mut Iw7Map, l: &walk::LodRec| -> Option<Vec<ModelSurface>> {
                                let (ms, data) = source(l)?;
                                let mut surfaces = Vec::new();
                                for (k, s) in ms.surfs.iter().enumerate() {
                                    let (Some(vs), Some(ts)) = (s.verts, s.tris) else { continue };
                                    let verts: Vec<[u8; 32]> = vs.get(data).chunks_exact(32).map(|c| c.try_into().unwrap()).collect();
                                    let tris: Vec<[u16; 3]> =
                                        ts.get(data).chunks_exact(6).map(|c| [u16_at(c, 0), u16_at(c, 2), u16_at(c, 4)]).collect();
                                    let material = material_of(out, sr.cap, x.materials.get(usize::from(l.surf_index) + k).copied().flatten());
                                    surfaces.push(ModelSurface { verts, tris, material });
                                }
                                Some(surfaces)
                            };
                            let Some(surfaces) = read(&mut out, found[0]) else { continue };
                            let coarse = match found.last() {
                                Some(l) if found.len() > 1 => read(&mut out, l).unwrap_or_default(),
                                _ => Vec::new(),
                            };
                            out.models.push(MapModel {
                                name: name.clone(),
                                mins: x.bounds.0,
                                maxs: x.bounds.1,
                                radius: x.radius,
                                surfaces,
                                coarse,
                            });
                            if si > 0 {
                                from_model_zones += 1;
                            }
                            built = Some(out.models.len() - 1);
                            break;
                        }
                        if built.is_none() && no_mesh.len() < 8 {
                            // Why: no XModel of that name, or none of its
                            // LODs' surfaces in the shared streams.
                            let why = match map.cap.xmodels.iter().find(|x| bare(&x.name) == name) {
                                None => "no xmodel".to_owned(),
                                Some(x) => format!(
                                    "lods {:?}",
                                    x.lods.iter().map(|l| (l.numsurfs, l.surfs.map(|s| bare(map.cap.name_of(s)).to_owned()))).collect::<Vec<_>>()
                                ),
                            };
                            no_mesh.push(format!("{name} ({why})"));
                        }
                        model_index.insert(name.clone(), built);
                        built
                    }
                };
                let Some(model) = model else { continue };
                let axis = [v3(d, D::placement__axis), v3(d, D::placement__axis + 12), v3(d, D::placement__axis + 24)];
                let origin = v3(d, D::placement__origin);
                let lighting_origin = g
                    .smodel_insts
                    .map(|at| v3(image, at + L::GfxStaticModelInst::SIZE * i + L::GfxStaticModelInst::lightingOrigin))
                    .filter(|p| p.iter().all(|c| c.is_finite()))
                    .unwrap_or(origin);
                let q = |v: f32| (v * 64.0).round() as i64;
                let key: [i64; 12] = std::array::from_fn(|k| if k < 3 { q(origin[k]) } else if k < 12 { q(axis[(k - 3) / 3][(k - 3) % 3] * 64.0) } else { 0 });
                if !placed_at.insert((model, key)) {
                    doubled += 1;
                    continue;
                }
                out.static_models.push(StaticModel {
                    model,
                    origin,
                    axis,
                    scale: f32_at(d, D::placement__scale),
                    cull_dist: f32::from(u16_at(d, D::cullDist)),
                    lighting_origin,
                    index: i as u32,
                });
            }
        }
        out.report.push(format!(
            "world: {} vertices, {} indices, {} surfaces, {} materials ({} not defined{}); {} static models placed of {}, {} meshes{}",
            out.vertices.len(),
            out.indices.len(),
            out.surfaces.len(),
            out.materials.len(),
            missing_materials.len(),
            if missing_materials.is_empty() { String::new() } else { format!(": {}", missing_materials.join(", ")) },
            out.static_models.len(),
            g.smodel_count,
            out.models.len(),
            if no_mesh.is_empty() { String::new() } else { format!("; no mesh for {}", no_mesh.join(", ")) },
        ));
        if doubled > 0 {
            out.report.push(format!("{doubled} placements left out: the same model again exactly where it already stands"));
        }
        if from_model_zones > 0 {
            out.report.push(format!("{from_model_zones} placed models' meshes from {} model zones", model_zones.len()));
        }

        // The baked light: each `GfxLightMap`'s images by name.
        // (A reference that named no slot, mp_junk's and mp_desert's among
        // them: the zone's own GfxLightMap of that index; a map zone holds
        // its world's light maps in order.)
        for (j, id) in g.light_maps.iter().enumerate() {
            let rec = match id {
                Some(id) => map.cap.light_maps.get(id.index as usize),
                None => map.cap.light_maps.get(j),
            };
            if let Some(l) = rec {
                out.light_maps.push(l.textures.map(|t| t.map(|t| bare(map.cap.name_of(t)).to_owned()).unwrap_or_default()));
            }
        }
        // (A world whose light map list is itself a reference: the zone's
        // light maps, in order.)
        if out.light_maps.is_empty() {
            for l in &map.cap.light_maps {
                out.light_maps.push(l.textures.map(|t| t.map(|t| bare(map.cap.name_of(t)).to_owned()).unwrap_or_default()));
            }
        }
        out.probe_image = g.images.iter().find(|(k, _)| *k == "reflectionProbeArray").map(|(_, id)| bare(map.cap.name_of(*id)).to_owned());
        out.probe_origins = g.probe_origins.clone();
        // The vision: the map's own, else the first.
        let own = format!("vision/{}.vision", out.name);
        let vision = map
            .cap
            .raw_files
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(&own))
            .or_else(|| map.cap.raw_files.iter().find(|r| r.name.starts_with("vision/") && r.name.contains(out.name.as_str())))
            .or_else(|| map.cap.raw_files.iter().find(|r| r.name.starts_with("vision/")));
        out.vision = vision
            .and_then(|r| Some((r.name.clone(), String::from_utf8_lossy(&raw_file(&map.cap, image, &r.name)?).into_owned())));
        out.report.push(format!(
            "light: {} light maps {:?}, vision {}",
            out.light_maps.len(),
            out.light_maps.iter().map(|l| l[1].as_str()).collect::<Vec<_>>(),
            out.vision.as_ref().map_or("none", |v| v.0.as_str())
        ));

        // Lights.
        for c in &map.cap.com_worlds {
            for l in &c.lights {
                out.lights.push(PrimaryLight { kind: l.kind, color: l.color, dir: l.dir, origin: l.origin, radius: l.radius });
            }
        }

        // Spawns and entities.
        let sstr = |i: u32| map.zone.assets.script_strings.get(i as usize).cloned().flatten().unwrap_or_default();
        if let Some(m) = map.cap.map_ents.first() {
            if let Some(s) = m.spawns {
                for r in s.get(image).chunks_exact(L::SpawnPointEntityRecord::SIZE) {
                    use L::SpawnPointEntityRecord as R;
                    out.spawns.push(SpawnPoint {
                        classname: sstr(u32_at(r, R::name)),
                        target: sstr(u32_at(r, R::target)),
                        script_noteworthy: sstr(u32_at(r, R::script_noteworthy)),
                        targetname: String::new(),
                        origin: v3(r, R::origin),
                        angles: v3(r, R::angles),
                    });
                }
            }
            out.entities = entity_text(&m.entity_string, &out.spawns);
        }
        // Scriptable instances.
        if let Some(m) = map.cap.map_ents.first()
            && let Some(span) = m.scriptables
        {
            use L::ScriptableInstance as I;
            for (i, r) in span.get(image).chunks_exact(I::SIZE).enumerate() {
                let name = |id: Option<&Option<AssetId>>| id.copied().flatten().map(|id| bare(map.cap.name_of(id)).to_owned()).unwrap_or_default();
                out.scriptables.push(ScriptableInstance {
                    def: name(m.scriptable_defs.get(i)),
                    model: name(m.scriptable_models.get(i)),
                    origin: v3(r, I::contextHeader__context__initialOrigin),
                    angles: v3(r, I::contextHeader__context__initialAngles),
                    targetname: sstr(u32_at(r, I::targetname)),
                });
            }
        }
        // Trigger volumes, brush models and path nodes (the Zombies layout
        // needs them).
        if let Some(m) = map.cap.map_ents.first()
            && let (Some(models), Some(hulls)) = (m.trigger_models, m.trigger_hulls)
        {
            use L::TriggerHull as H;
            use L::TriggerModel as TM;
            let hulls = hulls.get(image);
            for t in models.get(image).chunks_exact(TM::SIZE) {
                let (first, n) = (usize::from(u16_at(t, TM::firstHull)), usize::from(u16_at(t, TM::hullCount)));
                let model: Vec<([f32; 3], [f32; 3])> = (first..first + n)
                    .map_while(|k| hulls.get(H::SIZE * k..H::SIZE * (k + 1)))
                    .map(|hl| (v3(hl, H::bounds__midPoint), v3(hl, H::bounds__halfSize)))
                    .collect();
                out.trigger_hulls.push(model);
            }
        }
        if let Some(at) = g.models {
            use L::GfxBrushModel as B;
            for i in 0..g.model_count as usize {
                let b = &image[at + B::SIZE * i..at + B::SIZE * (i + 1)];
                let (mid, half) = (v3(b, B::bounds__midPoint), v3(b, B::bounds__halfSize));
                out.brush_models.push(BrushModel {
                    first_surface: u32_at(b, B::startSurfIndex),
                    surface_count: u32_at(b, B::surfaceCount),
                    mins: std::array::from_fn(|k| mid[k] - half[k]),
                    maxs: std::array::from_fn(|k| mid[k] + half[k]),
                });
            }
        }
        out.place_brush_models();
        if let Some(p) = map.cap.path_datas.first()
            && let Some(nodes) = p.nodes
        {
            use L::pathlink_s as K;
            use L::pathnode_t as N;
            for (i, n) in nodes.get(image).chunks_exact(N::SIZE).enumerate() {
                let links = p.links.get(i).copied().flatten().map_or(&[][..], |s| s.get(image));
                out.path_nodes.push(PathNode {
                    kind: u16_at(n, N::constant__type),
                    spawnflags: u16_at(n, N::constant__spawnflags),
                    targetname: sstr(u32_at(n, N::constant__targetname)),
                    target: sstr(u32_at(n, N::constant__target)),
                    script_noteworthy: sstr(u32_at(n, N::constant__script_noteworthy)),
                    script_linkname: sstr(u32_at(n, N::constant__script_linkName)),
                    animscript: sstr(u32_at(n, N::constant__animscript)),
                    origin: v3(n, N::constant__vLocalOrigin),
                    yaw: f32_at(n, N::constant__orientation__yaw_orient__fLocalAngle),
                    links: links
                        .chunks_exact(K::SIZE)
                        .map(|k| (u16_at(k, K::nodeNum), f32_at(k, K::fDist), k[K::negotiationLink] != 0))
                        .collect(),
                });
            }
        }
        out.report.push(format!(
            "entities: {} spawn points, {} trigger models, {} brush models, {} path nodes ({} links)",
            out.spawns.len(),
            out.trigger_hulls.len(),
            out.brush_models.len(),
            out.path_nodes.len(),
            out.path_nodes.iter().map(|n| n.links.len()).sum::<usize>()
        ));
        if let Some(sh) = &out.shared {
            out.report.push(format!("common zones: {} materials taken from {:?}", out.shared_materials, sh.zones));
        }
        if out.name.starts_with("cp_") {
            let layout = crate::zombies::ZmLayout::read(&out);
            out.report.push(format!("zombies layout: {}", layout.summary()));
        }
        Ok(out)
    }

    /// The brush models (`*<n>`, n > 0) the map's `script_brushmodel`
    /// entities show as it starts, moved into place.
    ///
    /// IW7 stores a brush model's surfaces about its own origin, in the
    /// world's surface list after the world's own (mp_riot's Geneva signs
    /// `window_decals`, its hardpoint grids; mp_flip's doors): drawn as they
    /// are, they stood at the world's origin. Those placed as the map starts
    /// are moved to their entity's origin and angles; the rest are left out
    /// (`WorldSurface::hidden_brush`): a later state (`hide_on_load`, a
    /// script's `show`, an `exploder`), a game mode's own (the hardpoint
    /// zones' grids: `hardpoint_zone_visual*`), one no entity names, and on
    /// a Zombies map (`cp_*`) every one (its doors and debris are its
    /// scripts' to open and clear).
    fn place_brush_models(&mut self) {
        let n = self.surfaces.len();
        let mut owner: Vec<Option<usize>> = vec![None; n];
        for (k, b) in self.brush_models.iter().enumerate().skip(1) {
            if b.first_surface == u32::MAX {
                continue;
            }
            let start = b.first_surface as usize;
            for o in owner.iter_mut().take((start + b.surface_count as usize).min(n)).skip(start) {
                *o = Some(k);
            }
        }
        let zombies = self.name.starts_with("cp_");
        let mut placed = vec![false; n];
        let mut moved = vec![false; self.vertices.len()];
        let mut count = 0usize;
        for e in parse_entities(&self.entities) {
            if zombies || !e.get("classname").is_some_and(|c| c.starts_with("script_brushmodel")) {
                continue;
            }
            let has = |k: &str, w: &str| e.get(k).is_some_and(|v| v.split([',', ' ']).any(|t| t == w));
            let targetname = e.get("targetname").map_or("", String::as_str);
            if targetname == "exploder" || targetname.starts_with("hardpoint_zone_visual") || has("script_noteworthy", "hide_on_load") || has("script_noteworthy", "show") {
                continue;
            }
            let Some(k) = e.get("model").and_then(|m| m.strip_prefix('*')).and_then(|m| m.parse::<usize>().ok()) else { continue };
            let Some(b) = self.brush_models.get(k).copied() else { continue };
            if k == 0 || b.first_surface == u32::MAX || b.surface_count == 0 {
                continue;
            }
            let vec3 = |key: &str| -> Option<[f32; 3]> {
                let v: Vec<f32> = e.get(key)?.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                (v.len() == 3).then(|| [v[0], v[1], v[2]])
            };
            let origin = vec3("origin").unwrap_or_default();
            let axis = axis_of(vec3("angles").unwrap_or_else(|| [0.0, e.get("angle").and_then(|a| a.parse().ok()).unwrap_or(0.0), 0.0]));
            let rot = |v: [f32; 3]| -> [f32; 3] { std::array::from_fn(|c| axis[0][c] * v[0] + axis[1][c] * v[1] + axis[2][c] * v[2]) };
            let start = b.first_surface as usize;
            for si in start..(start + b.surface_count as usize).min(n) {
                if std::mem::replace(&mut placed[si], true) {
                    continue;
                }
                count += 1;
                let s = &mut self.surfaces[si];
                let verts = s.first_vertex as usize..(s.first_vertex as usize + usize::from(s.vertex_count)).min(self.vertices.len());
                for vi in verts.clone() {
                    if std::mem::replace(&mut moved[vi], true) {
                        continue;
                    }
                    let v = &mut self.vertices[vi];
                    let p = rot(v.xyz);
                    v.xyz = std::array::from_fn(|c| origin[c] + p[c]);
                    v.normal = rot(v.normal);
                    v.tangent = rot(v.tangent);
                }
                let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                for v in &self.vertices[verts] {
                    for c in 0..3 {
                        lo[c] = lo[c].min(v.xyz[c]);
                        hi[c] = hi[c].max(v.xyz[c]);
                    }
                }
                if lo[0] <= hi[0] {
                    (s.mins, s.maxs) = (lo, hi);
                }
            }
        }
        let mut hidden = 0usize;
        for (i, s) in self.surfaces.iter_mut().enumerate() {
            s.brush_model = owner[i].map_or(0, |k| k as u32);
            s.hidden_brush = owner[i].is_some() && !placed[i];
            hidden += usize::from(s.hidden_brush);
        }
        self.report.push(format!("brush models: {count} surfaces placed by their entities, {hidden} not shown as the map starts"));
    }

    /// An image's pixels at the largest stream whose top level is at most
    /// `max_dim` (the smallest stream when none is), with the levels below.
    pub fn image(&self, name: &str, max_dim: u32) -> Result<ImageLevels, String> {
        let shared;
        let src = match self.images.get(bare(name)) {
            Some(src) => src,
            None => {
                // A picture of the common zones, else one of the engine's.
                match self.shared.as_ref().and_then(|s| Some((s.image_rec(name)?.clone(), s.inline(name).map(<[u8]>::to_vec)))) {
                    Some((rec, inline)) => {
                        shared = ImageSource { rec, inline };
                        &shared
                    }
                    None => return crate::shared::builtin(name).ok_or_else(|| format!("{name}: not in the map's zones")),
                }
            }
        };
        let im = &src.rec;
        if let Some(px) = &src.inline {
            return Ok(ImageLevels {
                format: im.format,
                width: u32::from(im.width),
                height: u32::from(im.height),
                levels: u32::from(im.level_count.max(1)),
                data: px.clone(),
                stream: None,
            });
        }
        let k = pick_stream(im, max_dim).ok_or_else(|| format!("{name}: no stream range"))?;
        let mut parts = vec![Vec::new(); k + 1];
        for (i, part) in parts.iter_mut().enumerate() {
            let Some(f) = im.stream_files[i].filter(|f| f.offset_end > f.offset) else { continue };
            let raw = read_image_range(&self.pak_dir, &f).map_err(|e| format!("{name}: {e}"))?;
            *part = texture::stream_part(im, i, &raw).map_err(|e| format!("{name} stream {i}: {e}"))?;
        }
        Ok(texture::stream_levels(im, k, &parts))
    }

    /// The `imagefile` ranges `image(name, max_dim)` reads.
    pub fn image_ranges(&self, name: &str, max_dim: u32) -> Vec<StreamFile> {
        let im = match self.images.get(bare(name)) {
            Some(src) if src.inline.is_some() => return Vec::new(),
            Some(src) => &src.rec,
            None => match self.shared.as_ref().and_then(|s| s.image_rec(name)) {
                Some(r) if r.pixels.is_none() => r,
                _ => return Vec::new(),
            },
        };
        let Some(k) = pick_stream(im, max_dim) else {
            return Vec::new();
        };
        (0..=k).filter_map(|i| im.stream_files[i].filter(|f| f.offset_end > f.offset)).collect()
    }

    /// The image record of a name (format, size, map type).
    pub fn image_info(&self, name: &str) -> Option<&ImageRec> {
        self.images.get(bare(name)).map(|s| &s.rec).or_else(|| self.shared.as_ref()?.image_rec(name))
    }
}

/// The stream of a streamed image to read for at most `max_dim`: the
/// largest whose top level fits, unless that is far smaller than asked (a
/// 4x4 tail stream) and a bigger one exists: then the smallest bigger one.
pub fn pick_stream(im: &ImageRec, max_dim: u32) -> Option<usize> {
    let usable = |k: usize| im.streams[k].0 != 0 && im.stream_files[k].is_some_and(|f| f.offset_end > f.offset);
    let dim = |k: usize| u32::from(im.streams[k].0.max(im.streams[k].1));
    let fits = (0..4).rev().find(|&k| usable(k) && dim(k) <= max_dim);
    let bigger = (0..4).find(|&k| usable(k) && dim(k) > max_dim);
    match (fits, bigger) {
        (Some(k), Some(b)) if dim(k) * 4 < max_dim => Some(b),
        (Some(k), _) => Some(k),
        (None, b) => b,
    }
}

/// The origins and angles of the structs named `default_player_start` in
/// a (named) entity string.
fn zombies_starts(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for block in text.split('{').skip(1) {
        let body = block.split('}').next().unwrap_or("");
        let get = |k: &str| {
            body.lines().find_map(|l| {
                let (key, v) = l.trim().split_once(' ')?;
                (key.trim_matches('"') == k).then(|| v.trim().trim_matches('"').to_owned())
            })
        };
        if get("targetname").as_deref() == Some("default_player_start")
            && let Some(origin) = get("origin")
        {
            out.push((origin, get("angles").unwrap_or_else(|| "0 0 0".to_owned())));
        }
    }
    out
}

/// A raw file of a walked zone by name (case-insensitive), inflated when
/// stored compressed (zlib).
pub fn raw_file(cap: &Capture, image: &[u8], name: &str) -> Option<Vec<u8>> {
    let r = cap.raw_files.iter().find(|r| r.name.eq_ignore_ascii_case(name))?;
    let data = r.data?.get(image);
    if r.compressed_len == 0 {
        return Some(data.get(..r.len as usize).unwrap_or(data).to_vec());
    }
    miniz_oxide::inflate::decompress_to_vec_zlib(data).ok()
}

/// An `imagefile` range: from the pak, or from a copy of just that range
/// (`iw7_imagefile<N>_<offset hex>.bin`) beside where the paks would be.
pub(crate) fn read_image_range(dir: &Path, f: &StreamFile) -> Result<Vec<u8>, String> {
    let pak = dir.join(format!("imagefile{}.pak", f.file_index));
    if pak.is_file() {
        return crate::pak::read_range(&pak, f).map_err(|e| format!("{}: {e}", pak.display()));
    }
    let one = dir.join(format!("iw7_imagefile{}_{:x}.bin", f.file_index, f.offset));
    std::fs::read(&one).map_err(|_| format!("{} not found", pak.display()))
}

/// An entity's axes (forward, left, up) from its pitch, yaw and roll in
/// degrees, as the engine's `AnglesToAxis`.
pub fn axis_of(angles: [f32; 3]) -> [[f32; 3]; 3] {
    let [p, y, r] = angles.map(f32::to_radians);
    let (sp, cp, sy, cy, sr, cr) = (p.sin(), p.cos(), y.sin(), y.cos(), r.sin(), r.cos());
    let forward = [cp * cy, cp * sy, -sp];
    let right = [-sr * sp * cy + cr * sy, -sr * sp * sy - cr * cy, -sr * cp];
    let up = [cr * sp * cy + sr * sy, cr * sp * sy - sr * cy, cr * cp];
    [forward, right.map(|v| -v), up]
}

/// A (named) entity string's entities as key/value maps.
pub fn parse_entities(text: &str) -> Vec<HashMap<String, String>> {
    let mut out = Vec::new();
    for block in text.split('{').skip(1) {
        let body = block.split('}').next().unwrap_or("");
        let mut e = HashMap::new();
        for l in body.lines() {
            if let Some((k, v)) = l.trim().split_once(' ') {
                e.insert(k.trim_matches('"').to_owned(), v.trim().trim_matches('"').to_owned());
            }
        }
        out.push(e);
    }
    out
}

/// The entity string with its keys named, and the spawn points added.
///
/// A Zombies map (`cp_*`) has no spawn list: its player starts are the
/// structs named `default_player_start`; each is also given as an
/// `info_player_start` (the start every rules set reads) and, for Black Ops
/// II's Zombies scripts, a struct named `initial_spawn_points`.
pub fn entity_text(raw: &str, spawns: &[SpawnPoint]) -> String {
    let names: HashMap<u32, &str> = ENTITY_KEYS.iter().copied().collect();
    let mut out = String::with_capacity(raw.len() + spawns.len() * 96);
    for line in raw.lines() {
        let t = line.trim();
        if t == "{" || t == "}" {
            out.push_str(t);
            out.push('\n');
            continue;
        }
        let Some((key, value)) = t.split_once(' ') else { continue };
        // A value is one quoted string; other shapes are left out.
        if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2 && value[1..value.len() - 1].find('"').is_none()) {
            continue;
        }
        let key = match key.parse::<u32>() {
            Ok(n) => names.get(&n).map_or_else(|| format!("_k{n}"), |s| (*s).to_owned()),
            Err(_) => key.trim_matches('"').to_owned(),
        };
        out.push_str(&format!("\"{key}\" {value}\n"));
    }
    let named = zombies_starts(&out);
    for (origin, angles) in named {
        for (classname, targetname) in [("info_player_start", ""), ("script_struct", "initial_spawn_points")] {
            out.push_str("{\n");
            out.push_str(&format!("\"classname\" \"{classname}\"\n\"origin\" \"{origin}\"\n\"angles\" \"{angles}\"\n"));
            if !targetname.is_empty() {
                out.push_str(&format!("\"targetname\" \"{targetname}\"\n"));
            }
            out.push_str("}\n");
        }
    }
    for s in spawns {
        out.push_str("{\n");
        out.push_str(&format!("\"classname\" \"{}\"\n", s.classname));
        out.push_str(&format!("\"origin\" \"{} {} {}\"\n", s.origin[0], s.origin[1], s.origin[2]));
        out.push_str(&format!("\"angles\" \"{} {} {}\"\n", s.angles[0], s.angles[1], s.angles[2]));
        for (k, v) in [("target", &s.target), ("script_noteworthy", &s.script_noteworthy)] {
            if !v.is_empty() {
                out.push_str(&format!("\"{k}\" \"{v}\"\n"));
            }
        }
        out.push_str("}\n");
    }
    out
}
