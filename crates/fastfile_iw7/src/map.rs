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
        Self { zone: zone.to_owned(), techsets, pak_dir: dir, shared_ranges: None, extra_zones: Vec::new() }
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
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    /// What was read and what was missing, line by line.
    pub report: Vec<String>,
    images: HashMap<String, ImageSource>,
    pak_dir: PathBuf,
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
fn shared_data(src: &MapSources, header: &container::FileHeader) -> Result<crate::pak::SharedVirtual, String> {
    let raw = match &src.shared_ranges {
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
        // Images by name: the map's first.
        for w in std::iter::once(&map).chain(zones.iter()) {
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
        for (z, w) in std::iter::once(&map).chain(zones.iter()).enumerate() {
            for (i, m) in w.cap.materials.iter().enumerate() {
                if !m.name.starts_with(',') {
                    material_defs.entry(m.name.clone()).or_insert((z, i));
                }
            }
        }
        let all: Vec<&Walked> = std::iter::once(&map).chain(zones.iter()).collect();
        let mut material_index: HashMap<String, Option<usize>> = HashMap::new();
        let mut missing_materials = Vec::new();
        let mut material_of = |out: &mut Iw7Map, cap: &Capture, id: Option<AssetId>| -> Option<usize> {
            let name = bare(cap.name_of(id?)).to_owned();
            if let Some(i) = material_index.get(&name) {
                return *i;
            }
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
        let shared = match shared_data(src, &map.zone.header) {
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
        let surfs_by_name: HashMap<&str, usize> = shared
            .as_ref()
            .map(|(_, c)| c.model_surfs.iter().enumerate().map(|(i, m)| (m.name.as_str(), i)).collect())
            .unwrap_or_default();
        let mut model_index: HashMap<String, Option<usize>> = HashMap::new();
        let mut no_mesh = Vec::new();
        if let Some(at) = g.smodels {
            use L::GfxStaticModelDrawInst as D;
            for i in 0..g.smodel_count as usize {
                let d = &image[at + D::SIZE * i..at + D::SIZE * (i + 1)];
                let Some(id) = g.smodel_models.get(i).copied().flatten() else { continue };
                let name = bare(map.cap.name_of(id)).to_owned();
                let model = match model_index.get(&name) {
                    Some(m) => *m,
                    None => {
                        let built = (|| {
                            let (sv, scap) = shared.as_ref()?;
                            let x = map.cap.xmodels.iter().find(|x| x.name == name)?;
                            // The LODs whose surfaces are in the shared streams: the
                            // first is drawn, the last is the collision's.
                            let found: Vec<&walk::LodRec> = x
                                .lods
                                .iter()
                                .filter(|l| l.numsurfs > 0 && l.surfs.is_some_and(|s| surfs_by_name.contains_key(bare(map.cap.name_of(s)))))
                                .collect();
                            let lod = *found.first()?;
                            let mut read = |out: &mut Iw7Map, l: &walk::LodRec| -> Option<Vec<ModelSurface>> {
                                let ms = &scap.model_surfs[surfs_by_name[bare(map.cap.name_of(l.surfs?))]];
                                let mut surfaces = Vec::new();
                                for (k, s) in ms.surfs.iter().enumerate() {
                                    let (Some(vs), Some(ts)) = (s.verts, s.tris) else { continue };
                                    let verts: Vec<[u8; 32]> = vs.get(&sv.data).chunks_exact(32).map(|c| c.try_into().unwrap()).collect();
                                    let tris: Vec<[u16; 3]> =
                                        ts.get(&sv.data).chunks_exact(6).map(|c| [u16_at(c, 0), u16_at(c, 2), u16_at(c, 4)]).collect();
                                    let material = material_of(out, &map.cap, x.materials.get(usize::from(l.surf_index) + k).copied().flatten());
                                    surfaces.push(ModelSurface { verts, tris, material });
                                }
                                Some(surfaces)
                            };
                            let surfaces = read(&mut out, lod)?;
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
                            Some(out.models.len() - 1)
                        })();
                        if built.is_none() && no_mesh.len() < 8 {
                            no_mesh.push(name.clone());
                        }
                        model_index.insert(name.clone(), built);
                        built
                    }
                };
                let Some(model) = model else { continue };
                let axis = [v3(d, D::placement__axis), v3(d, D::placement__axis + 12), v3(d, D::placement__axis + 24)];
                out.static_models.push(StaticModel {
                    model,
                    origin: v3(d, D::placement__origin),
                    axis,
                    scale: f32_at(d, D::placement__scale),
                    cull_dist: f32::from(u16_at(d, D::cullDist)),
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
        out.report.push(format!("entities: {} spawn points", out.spawns.len()));
        if out.name.starts_with("cp_") {
            let census: Vec<String> = zombies_census(&out.entities).into_iter().map(|(what, n)| format!("{what} {n}")).collect();
            out.report.push(format!("zombies layout: {}", census.join(", ")));
        }
        Ok(out)
    }

    /// An image's pixels at the largest stream whose top level is at most
    /// `max_dim` (the smallest stream when none is), with the levels below.
    pub fn image(&self, name: &str, max_dim: u32) -> Result<ImageLevels, String> {
        let src = self.images.get(bare(name)).ok_or_else(|| format!("{name}: not in the map's zones"))?;
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
        let Some(src) = self.images.get(bare(name)) else { return Vec::new() };
        if src.inline.is_some() {
            return Vec::new();
        }
        let im = &src.rec;
        let Some(k) = pick_stream(im, max_dim) else {
            return Vec::new();
        };
        (0..=k).filter_map(|i| im.stream_files[i].filter(|f| f.offset_end > f.offset)).collect()
    }

    /// The image record of a name (format, size, map type).
    pub fn image_info(&self, name: &str) -> Option<&ImageRec> {
        self.images.get(bare(name)).map(|s| &s.rec)
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

/// What an Infinite Warfare Zombies map places for its game, by the names
/// its scripts look them up by: (what, count). For the report, and for
/// whoever maps them onto Black Ops II's Zombies structs.
pub fn zombies_census(entities: &str) -> Vec<(&'static str, usize)> {
    let mut by: HashMap<String, usize> = HashMap::new();
    for block in entities.split('{').skip(1) {
        let body = block.split('}').next().unwrap_or("");
        for l in body.lines() {
            if let Some((k, v)) = l.trim().split_once(' ')
                && matches!(k.trim_matches('"'), "targetname" | "script_noteworthy")
            {
                *by.entry(v.trim().trim_matches('"').to_owned()).or_default() += 1;
            }
        }
    }
    let n = |names: &[&str]| names.iter().map(|k| by.get(*k).copied().unwrap_or(0)).sum::<usize>();
    vec![
        ("player starts (default_player_start)", n(&["default_player_start"])),
        ("player respawns (player_respawn_loc)", n(&["player_respawn_loc"])),
        ("windows / barriers (window_entrance)", n(&["window_entrance"])),
        ("buyable doors and debris (door_buy, debris_*)", n(&["door_buy", "debris_750", "debris_1000", "debris_1250", "debris_1500", "debris_2000"])),
        ("team doors (team_door_switch)", n(&["team_door_switch"])),
        ("magic wheel spots (zmb_magic_wheel_spinner / spinner)", n(&["spinner"])),
        ("power switches (generator)", n(&["generator"])),
        ("crafting stations", n(&["crafting_station"])),
        ("fast travel portals", n(&["fast_travel"])),
        ("spawn volumes (spawn_volume)", n(&["spawn_volume"])),
        ("zone volumes (mini_zone)", n(&["mini_zone"])),
        ("zombie spawn spots (ground / wall / ceiling)", n(&["low", "medium", "high"])),
        ("dog spawners", n(&["dog_spawner"])),
    ]
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
