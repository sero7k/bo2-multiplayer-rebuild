//! A Black Ops III map zone (`mp_*.ff`, `zm_*.ff`) read whole, in plain
//! terms a converter can use: the drawn world (vertices, indices, surfaces
//! and their materials), the placed static models, the materials with their
//! image names (pixels from the zone or the `.xpak` packages, `texture`),
//! the collision (brushes, BSP, triangle mesh), the entities and the sun.
//!
//! The zone's walk (`walk`) keeps file offsets of the map assets' arrays
//! (`walk::world`, `walk::gfxworld`); this reads them.
//!
//! World vertices (read on `mp_nuketown_x`): positions (12 bytes) in one
//! buffer, attributes (20 bytes: colour, UV as two floats, normal and
//! tangent as unsigned 10:10:10 over 1023) in another; a surface's
//! vertices start at its position offset / 12, its triangles index the
//! index buffer from its base index, relative to that start. Several
//! surfaces share a vertex range: those have a vertex count of 0 (the count
//! is then the largest index + 1).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::texture::{self, ImageLevels};
use crate::walk::capture::{Capture, ImageRec};
use crate::walk::world::ClipMapRec;
use crate::xpak::XpakSet;

/// One world vertex, unpacked.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldVertex {
    pub xyz: [f32; 3],
    /// RGBA bytes as stored.
    pub color: u32,
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
}

/// Black Ops III's packed unit vector: unsigned 10:10:10 over 1023.
pub fn unpack_unit(packed: u32) -> [f32; 3] {
    std::array::from_fn(|k| ((packed >> (10 * k)) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0)
}

/// One world surface: its triangles index the index buffer from
/// `base_index`, relative to `first_vertex`.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldSurface {
    pub first_vertex: u32,
    pub vertex_count: u32,
    pub tri_count: u16,
    pub base_index: u32,
    pub material: Option<usize>,
    pub flags: u8,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
}

/// A material: its technique set's name and its textures (name hash, image).
#[derive(Clone, Debug, Default)]
pub struct MapMaterial {
    pub name: String,
    pub technique_set: String,
    pub textures: Vec<(u32, String)>,
}

/// One model surface: positions, normals, UVs and triangles.
#[derive(Clone, Debug, Default)]
pub struct ModelSurface {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tris: Vec<[u16; 3]>,
    pub material: Option<usize>,
}

/// A static model's first LOD (and its coarsest, for collision).
#[derive(Clone, Debug, Default)]
pub struct MapModel {
    pub name: String,
    pub surfaces: Vec<ModelSurface>,
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
}

/// A static model the collision lists (`cStaticModel_t`): its model's name
/// and its placement (axes and scale from the inverse scaled axes).
#[derive(Clone, Debug, Default)]
pub struct ClipStaticModel {
    pub name: String,
    pub contents: u32,
    pub origin: [f32; 3],
    /// Rows: the model's forward, left and up axes in the world.
    pub axis: [[f32; 3]; 3],
    pub scale: f32,
}

/// A brush sub-model (`*<n>`): its world surfaces and its bounds.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrushModel {
    pub first_surface: u32,
    pub surface_count: u32,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
}

/// A collision brush: its planes (normal, distance) with each plane's
/// surface flags, the six axial ones first (+x, -x, +y, -y, +z, -z).
#[derive(Clone, Debug, Default)]
pub struct ClipBrush {
    pub contents: u32,
    pub planes: Vec<[f32; 4]>,
    pub plane_surface_flags: Vec<u32>,
}

/// A BSP node: its plane and children (negative: leaf `-1 - n`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipNode {
    pub plane: [f32; 4],
    pub children: [i32; 2],
}

/// A BSP leaf: its brushes and its triangle trees.
#[derive(Clone, Debug, Default)]
pub struct ClipLeaf {
    pub brushes: Vec<u16>,
    pub first_coll_aabb: u32,
    pub coll_aabb_count: u32,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
}

/// A collision partition: `tri_count` triangles from `first_tri`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipPartition {
    pub tri_count: u8,
    pub first_tri: u32,
}

/// A collision AABB tree node (`u`: first child, or the partition of a leaf).
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipAabb {
    pub origin: [f32; 3],
    pub half_size: [f32; 3],
    pub material: u16,
    pub child_count: u16,
    pub u: u32,
}

/// A collision sub-model (`*<n>`; 0 is the world).
#[derive(Clone, Debug, Default)]
pub struct ClipModel {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub radius: f32,
    pub leaf: ClipLeaf,
}

/// The map's collision as the zone has it.
#[derive(Clone, Debug, Default)]
pub struct MapClip {
    /// (name, surface flags, content flags).
    pub materials: Vec<(String, u32, u32)>,
    pub brushes: Vec<ClipBrush>,
    pub nodes: Vec<ClipNode>,
    pub leafs: Vec<ClipLeaf>,
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    /// One bit per triangle edge (`3 * tri + k`).
    pub walkable: Vec<u8>,
    pub partitions: Vec<ClipPartition>,
    pub aabb_trees: Vec<ClipAabb>,
    pub models: Vec<ClipModel>,
    /// The static models that collide.
    pub static_models: Vec<ClipStaticModel>,
}

/// The sun (the first sun volume's first state): pitch and yaw (degrees),
/// colour and intensity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sun {
    pub pitch: f32,
    pub yaw: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

/// Where a map's files are.
#[derive(Clone, Debug)]
pub struct MapSources {
    pub zone: PathBuf,
    /// The folder of the `.xpak` packages (the zone's folder).
    pub pak_dir: PathBuf,
}

impl MapSources {
    pub fn beside(zone: &Path) -> Self {
        Self { zone: zone.to_owned(), pak_dir: zone.parent().map(Path::to_path_buf).unwrap_or_default() }
    }
}

/// A map, read.
#[derive(Default)]
pub struct T7Map {
    pub name: String,
    pub vertices: Vec<WorldVertex>,
    pub indices: Vec<u16>,
    pub surfaces: Vec<WorldSurface>,
    pub materials: Vec<MapMaterial>,
    pub models: Vec<MapModel>,
    pub static_models: Vec<StaticModel>,
    /// The entity string as the zone has it.
    pub entities: String,
    /// The trigger models (`"model" "?N"`): each its hulls (middle, half
    /// size).
    pub trigger_models: Vec<Vec<([f32; 3], [f32; 3])>>,
    pub brush_models: Vec<BrushModel>,
    pub clip: Option<MapClip>,
    pub sun: Option<Sun>,
    /// The sky box models (first sun volume).
    pub sky_models: Vec<String>,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    /// What was read and what was missing, line by line.
    pub report: Vec<String>,
    /// (bo2mp: surfaces drawn but left out of the collision built from the
    /// drawn world, by index; empty: none. Set by the Modern Warfare III
    /// lane for a big map's sea sheets and far backdrop.)
    pub no_collision: Vec<bool>,
    images: HashMap<String, ImageRec>,
    /// (bo2mp: pictures another game's reader decoded for its materials,
    /// by name: [`Self::add_image`].)
    added_images: HashMap<String, ImageLevels>,
    zone_image: Vec<u8>,
    paks: Option<XpakSet>,
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
fn u64_at(b: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}
fn v3(b: &[u8], o: usize) -> [f32; 3] {
    [f32_at(b, o), f32_at(b, o + 4), f32_at(b, o + 8)]
}

fn bare(name: &str) -> &str {
    name.trim_start_matches(',')
}

/// The packages a map's pictures and meshes can be in, opened once per
/// folder (every `*.xpak` there: base, patches, the map's own).
fn packages(dir: &Path) -> Result<XpakSet, String> {
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, XpakSet>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(set) = cache.lock().map_err(|_| "xpak cache poisoned".to_owned())?.get(dir) {
        return Ok(set.clone());
    }
    let (set, _bad) = XpakSet::open_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    cache.lock().map_err(|_| "xpak cache poisoned".to_owned())?.insert(dir.to_owned(), set.clone());
    Ok(set)
}

impl T7Map {
    /// Reads a map. Errors only when the zone cannot be read or has no
    /// world; what is missing otherwise is in `report`.
    pub fn load(src: &MapSources) -> Result<Self, String> {
        let zone = crate::Zone::open(&src.zone).map_err(|e| format!("{}: {e}", src.zone.display()))?;
        let w = crate::walk::walk(&zone);
        let mut out = T7Map {
            name: src.zone.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
            ..Default::default()
        };
        match &w.stop {
            crate::walk::Stop::Done { .. } => out.report.push(format!("zone: walked to its end ({} assets)", w.walked.len())),
            other => out.report.push(format!("zone: walk stopped ({other:?}); reading what was walked")),
        }
        let cap = w.capture;
        let image = &zone.image;
        out.read_world(&cap, image)?;
        out.read_models(&cap, image, src);
        out.read_clip(&cap, image);
        if let Some(m) = cap.world.map_ents.first()
            && let Some((at, n)) = m.entity_string
        {
            let raw = &image[at..at + n];
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            out.entities = String::from_utf8_lossy(&raw[..end]).into_owned();
        }
        if let Some(m) = cap.world.map_ents.first()
            && let (Some((mat, mn)), Some((hat, hn))) = (m.trigger_models, m.trigger_hulls)
        {
            // TriggerModel (8: contents, u16 hull count, u16 first hull);
            // TriggerHull (32: middle, half size, contents, slabs).
            let image = &zone.image;
            for i in 0..mn {
                let at = mat + 8 * i;
                let (count, first) = (u16_at(image, at + 4) as usize, u16_at(image, at + 6) as usize);
                out.trigger_models.push(
                    (first..(first + count).min(hn)).map(|h| (v3(image, hat + 32 * h), v3(image, hat + 32 * h + 12))).collect(),
                );
            }
        }
        out.report.push(format!(
            "entities: {} bytes, {} entities, {} trigger models",
            out.entities.len(),
            out.entities.matches('{').count(),
            out.trigger_models.len()
        ));
        for im in &cap.images {
            if !im.name.starts_with(',') {
                out.images.entry(im.name.clone()).or_insert_with(|| im.clone());
            }
        }
        out.zone_image = zone.image;
        match packages(&src.pak_dir) {
            Ok(p) => out.paks = Some(p),
            Err(e) => out.report.push(format!("packages: {e} (streamed pictures not drawn)")),
        }
        Ok(out)
    }

    fn material_index(&mut self, cap: &Capture, index: &mut HashMap<String, Option<usize>>, name: &str) -> Option<usize> {
        let name = bare(name);
        if let Some(i) = index.get(name) {
            return *i;
        }
        let found = cap.material(name).or_else(|| cap.material(&format!(",{name}"))).map(|m| {
            self.materials.push(MapMaterial {
                name: name.to_owned(),
                technique_set: m.techset.as_deref().map(bare).unwrap_or_default().to_owned(),
                textures: m.textures.iter().map(|(h, i)| (*h, bare(i).to_owned())).collect(),
            });
            self.materials.len() - 1
        });
        index.insert(name.to_owned(), found);
        found
    }

    fn read_world(&mut self, cap: &Capture, image: &[u8]) -> Result<(), String> {
        use crate::walk::gfxworld::{BRUSH_MODEL, SURFACE};
        let g = cap.world.gfx_worlds.first().ok_or("no GfxWorld in the zone")?;
        let mut materials: HashMap<String, Option<usize>> = HashMap::new();
        let (Some((pos, pos_len)), Some((att, att_len))) = (g.positions, g.attributes) else {
            return Err("GfxWorld: no vertex data in the zone".into());
        };
        let n = (pos_len / 12).min(att_len / 20);
        self.vertices = (0..n)
            .map(|i| {
                let a = att + 20 * i;
                WorldVertex {
                    xyz: v3(image, pos + 12 * i),
                    color: u32_at(image, a),
                    uv: [f32_at(image, a + 4), f32_at(image, a + 8)],
                    normal: unpack_unit(u32_at(image, a + 12)),
                    tangent: unpack_unit(u32_at(image, a + 16)),
                }
            })
            .collect();
        if let Some((at, n)) = g.indices {
            self.indices = (0..n).map(|i| u16_at(image, at + 2 * i)).collect();
        }
        let mut no_material = 0;
        if let Some((at, n)) = g.surfaces {
            for i in 0..n {
                let s = at + SURFACE * i;
                let first = u32_at(image, s + 12) / 12;
                let tri_count = u16_at(image, s + 42);
                let base_index = u32_at(image, s + 44);
                let mut vertex_count = u32::from(u16_at(image, s + 40));
                if vertex_count == 0 {
                    let tris = self.indices.get(base_index as usize..(base_index as usize + 3 * tri_count as usize).min(self.indices.len()));
                    vertex_count = tris.and_then(|t| t.iter().max()).map_or(0, |&m| u32::from(m) + 1);
                }
                let material = g.surface_materials.get(i).cloned().flatten().and_then(|m| self.material_index(cap, &mut materials, &m));
                if material.is_none() {
                    no_material += 1;
                }
                self.surfaces.push(WorldSurface {
                    first_vertex: first,
                    vertex_count,
                    tri_count,
                    base_index,
                    material,
                    flags: image[s + 80],
                    mins: v3(image, s + 48),
                    maxs: v3(image, s + 60),
                });
            }
        }
        let h = g.header_at;
        self.mins = v3(image, h + 0x2c0);
        self.maxs = v3(image, h + 0x2cc);
        if let Some((at, n)) = g.brush_models {
            for i in 0..n {
                let b = at + BRUSH_MODEL * i;
                self.brush_models.push(BrushModel {
                    mins: v3(image, b + 32),
                    maxs: v3(image, b + 44),
                    surface_count: u32_at(image, b + 56),
                    first_surface: u32_at(image, b + 60),
                });
            }
        }
        if g.sun_at != 0 {
            // GfxConfig_Sun: settings[0] at 60 (pitch, yaw, colour, intensity).
            let st = g.sun_at + 60;
            self.sun = Some(Sun {
                pitch: f32_at(image, st),
                yaw: f32_at(image, st + 4),
                color: v3(image, st + 8),
                intensity: f32_at(image, st + 20),
            });
        }
        self.sky_models = g.sky_models.iter().map(|m| bare(m).to_owned()).collect();
        let mut mean = [0u64; 4];
        for v in &self.vertices {
            for (k, m) in mean.iter_mut().enumerate() {
                *m += u64::from((v.color >> (8 * k)) & 0xff);
            }
        }
        let n = self.vertices.len().max(1) as u64;
        self.report.push(format!("world vertex colour: mean bytes {:?}", mean.map(|m| m / n)));
        self.report.push(format!(
            "world: {} vertices, {} indices, {} surfaces ({} without a material), {} materials, {} brush models, bounds {:?}..{:?}",
            self.vertices.len(),
            self.indices.len(),
            self.surfaces.len(),
            no_material,
            self.materials.len(),
            self.brush_models.len(),
            self.mins,
            self.maxs
        ));
        Ok(())
    }

    /// The static models: each distinct model's first LOD (and coarsest),
    /// decoded by `mesh`; placements from the draw instances.
    fn read_models(&mut self, cap: &Capture, image: &[u8], src: &MapSources) {
        use crate::walk::gfxworld::SMODEL_DRAW;
        let Some(g) = cap.world.gfx_worlds.first() else { return };
        let Some((at, n)) = g.smodels else { return };
        let paks = packages(&src.pak_dir).ok();
        let mut materials: HashMap<String, Option<usize>> = self.materials.iter().enumerate().map(|(i, m)| (m.name.clone(), Some(i))).collect();
        let mut built: HashMap<String, Option<usize>> = HashMap::new();
        let mut failures: Vec<String> = Vec::new();
        for i in 0..n {
            let d = at + SMODEL_DRAW * i;
            let Some(name) = g.smodel_models.get(i).cloned().flatten() else { continue };
            let name = bare(&name).to_owned();
            let model = match built.get(&name) {
                Some(m) => *m,
                None => {
                    let r = self.build_model(cap, image, paks.as_ref(), &mut materials, &name);
                    let m = match r {
                        Ok(m) => Some(m),
                        Err(e) => {
                            if failures.len() < 8 {
                                failures.push(format!("{name}: {e}"));
                            }
                            None
                        }
                    };
                    built.insert(name.clone(), m);
                    m
                }
            };
            let Some(model) = model else { continue };
            self.static_models.push(StaticModel {
                model,
                origin: v3(image, d),
                axis: [v3(image, d + 12), v3(image, d + 24), v3(image, d + 36)],
                scale: f32_at(image, d + 48),
            });
        }
        self.report.push(format!(
            "static models: {} placed of {}, {} models built of {}{}",
            self.static_models.len(),
            n,
            self.models.len(),
            built.len(),
            if failures.is_empty() { String::new() } else { format!("; not built: {}", failures.join("; ")) }
        ));
    }

    fn build_model(
        &mut self,
        cap: &Capture,
        image: &[u8],
        paks: Option<&XpakSet>,
        materials: &mut HashMap<String, Option<usize>>,
        name: &str,
    ) -> Result<usize, String> {
        let x = cap.model(name).or_else(|| cap.model(&format!(",{name}"))).ok_or("not in the zone")?;
        let mut lods = Vec::new();
        for (k, lod) in x.lods.iter().enumerate() {
            let Some(me) = cap.mesh(bare(&lod.mesh)).or_else(|| cap.mesh(&lod.mesh)) else { continue };
            let data = match (me.data, me.xpak) {
                (Some((at, n)), _) => image.get(at..at + n).map(<[u8]>::to_vec),
                (None, Some(e)) => paks.and_then(|p| p.read(e.key).ok()),
                _ => None,
            };
            let Some(data) = data else { continue };
            let geo = crate::mesh::decode(me, &data).map_err(|e| e.to_string())?;
            let mut surfaces = Vec::new();
            for (si, s) in geo.surfaces.iter().enumerate() {
                let mut remap: HashMap<u32, u16> = HashMap::new();
                let mut sur = ModelSurface::default();
                for t in s.indices.chunks_exact(3) {
                    let mut tri = [0u16; 3];
                    for (j, &v) in t.iter().enumerate() {
                        let next = remap.len() as u16;
                        tri[j] = *remap.entry(v).or_insert_with(|| {
                            let v = v as usize;
                            sur.positions.push(geo.positions.get(v).copied().unwrap_or_default());
                            sur.normals.push(geo.normals.get(v).copied().unwrap_or([0.0, 0.0, 1.0]));
                            sur.uvs.push(geo.uvs.get(v).copied().unwrap_or_default());
                            next
                        });
                    }
                    sur.tris.push(tri);
                }
                sur.material = lod.materials.get(si).and_then(|m| self.material_index(cap, materials, m));
                surfaces.push(sur);
            }
            lods.push((k, surfaces));
        }
        let mut lods = lods.into_iter();
        let (_, surfaces) = lods.next().ok_or("no LOD mesh with data")?;
        let coarse = lods.last().map(|(_, s)| s).unwrap_or_default();
        self.models.push(MapModel { name: name.to_owned(), surfaces, coarse });
        Ok(self.models.len() - 1)
    }

    fn read_clip(&mut self, cap: &Capture, image: &[u8]) {
        let Some(c) = cap.world.clip_maps.first() else {
            self.report.push("collision: no clip map".into());
            return;
        };
        match read_clip(c, image) {
            Ok(clip) => {
                self.report.push(format!(
                    "collision: {} brushes, {} nodes, {} leafs, {} vertices, {} triangles, {} partitions, {} AABB nodes, {} sub-models, {} materials (references {:+#x} off)",
                    clip.brushes.len(),
                    clip.nodes.len(),
                    clip.leafs.len(),
                    clip.verts.len(),
                    clip.tris.len(),
                    clip.partitions.len(),
                    clip.aabb_trees.len(),
                    clip.models.len(),
                    clip.materials.len(),
                    c.virt_delta
                ));
                self.clip = Some(clip);
            }
            Err(e) => self.report.push(format!("collision: {e}")),
        }
    }

    /// An image's pixels: the largest streamed part whose top level is at
    /// most `max_dim` (its single level; the smallest part's whole chain
    /// when that is the one), or the zone's own pixels.
    pub fn image(&self, name: &str, max_dim: u32) -> Result<ImageLevels, String> {
        if let Some(l) = self.added_images.get(name) {
            return Ok(l.clone());
        }
        let im = self.images.get(bare(name)).ok_or_else(|| format!("{name}: not in the zone"))?;
        if im.xpak.is_empty() {
            return texture::zone_levels(im, &self.zone_image).map_err(|e| format!("{name}: {e}"));
        }
        let paks = self.paks.as_ref().ok_or_else(|| format!("{name}: no packages"))?;
        let dim = |k: usize| u32::from(im.xpak[k].width.max(im.xpak[k].height));
        let k = (0..im.xpak.len()).rev().find(|&k| dim(k) <= max_dim).unwrap_or(0);
        let p = &im.xpak[k];
        let data = paks.read(p.key).map_err(|e| format!("{name}: {e}"))?;
        let (w, h) = (u32::from(p.width), u32::from(p.height));
        let levels = if k == 0 { texture::full_levels(w, h) } else { 1 };
        let want = texture::chain_size(im.format, w, h, levels).ok_or_else(|| format!("{name}: format {} not handled", im.format))?;
        if data.len() < want {
            return Err(format!("{name}: part {k} decodes to {:#x} bytes, expected {want:#x}", data.len()));
        }
        Ok(ImageLevels { format: im.format, width: w, height: h, levels, data: data[..want].to_vec(), part: Some(k) })
    }

    /// (bo2mp: a picture another game's reader decoded, for a material of
    /// its own to name; [`Self::image`] hands it out as it is.)
    pub fn add_image(&mut self, name: String, levels: ImageLevels) {
        self.added_images.insert(name, levels);
    }

    /// The image record of a name (format, size).
    pub fn image_info(&self, name: &str) -> Option<&ImageRec> {
        self.images.get(bare(name))
    }
}

/// The clip map's arrays as plain records.
fn read_clip(c: &ClipMapRec, image: &[u8]) -> Result<MapClip, String> {
    use crate::walk::world::{CLIP_BRUSH, CLIP_CMODEL, CLIP_LEAF, LEAF_BRUSH_NODE};
    let mut out = MapClip {
        materials: c.materials.iter().map(|(n, s, f)| (n.clone(), *s as u32, *f as u32)).collect(),
        ..Default::default()
    };
    let (pat, pn) = c.planes.ok_or("no planes")?;
    let plane = |i: usize| -> [f32; 4] {
        let p = pat + 20 * i.min(pn.saturating_sub(1));
        [f32_at(image, p), f32_at(image, p + 4), f32_at(image, p + 8), f32_at(image, p + 12)]
    };
    // Brushes: the six axial planes from the bounds, then the sides.
    if let Some((at, n)) = c.brushes {
        let mut unresolved = 0;
        for i in 0..n {
            let b = at + CLIP_BRUSH * i;
            let (mins, maxs) = (v3(image, b), v3(image, b + 16));
            let contents = u32_at(image, b + 12);
            let mut planes =
                vec![[1.0, 0.0, 0.0, maxs[0]], [-1.0, 0.0, 0.0, -mins[0]], [0.0, 1.0, 0.0, maxs[1]], [0.0, -1.0, 0.0, -mins[1]], [0.0, 0.0, 1.0, maxs[2]], [
                    0.0, 0.0, -1.0, -mins[2],
                ]];
            // axial_sflags[2][3] at 64: [min x, y, z], [max x, y, z].
            let sf: [u32; 6] = std::array::from_fn(|k| u32_at(image, b + 64 + 4 * k));
            let mut flags = vec![sf[3], sf[0], sf[4], sf[1], sf[5], sf[2]];
            let sides = u32_at(image, b + 28) as usize;
            match c.brush_side_lists.get(i).copied().flatten() {
                Some(list) => {
                    for k in 0..sides {
                        let side = list + 16 * k;
                        match c.plane_index(u64_at(image, side)) {
                            Some(p) => {
                                planes.push(plane(p));
                                flags.push(u32_at(image, side + 12));
                            }
                            None => unresolved += 1,
                        }
                    }
                }
                None if sides > 0 => unresolved += sides,
                None => {}
            }
            out.brushes.push(ClipBrush { contents, planes, plane_surface_flags: flags });
        }
        if unresolved > 0 {
            return Err(format!("{unresolved} brush sides whose plane could not be resolved"));
        }
    }
    // Leaf brush lists, flattened per leaf (the node tree under a leaf).
    let nodes_at = c.leaf_brush_nodes;
    let leaf_brushes = |root: usize| -> Vec<u16> {
        let mut out = Vec::new();
        let Some((at, n)) = nodes_at else { return out };
        let mut pending = vec![(root, 0usize)];
        while let Some((i, depth)) = pending.pop() {
            if i >= n || depth > 64 {
                continue;
            }
            let p = at + LEAF_BRUSH_NODE * i;
            let count = u16_at(image, p + 2) as i16;
            if count > 0 {
                if let Some((list, k)) = c.leaf_brush_lists.get(i).copied().flatten() {
                    out.extend((0..k).map(|j| u16_at(image, list + 2 * j)));
                }
            } else {
                // (As Black Ops and Black Ops II: a negative count also visits
                // the next node; the offsets are relative to this one.)
                if count < 0 {
                    pending.push((i + 1, depth + 1));
                }
                let (a, b) = (u16_at(image, p + 16) as usize, u16_at(image, p + 18) as usize);
                if a > 0 {
                    pending.push((i + a, depth + 1));
                }
                if b > 0 {
                    pending.push((i + b, depth + 1));
                }
            }
        }
        out
    };
    let leaf_at = |p: usize, node_at: usize| -> ClipLeaf {
        let node = u64_at(image, p + node_at) as i64;
        ClipLeaf {
            brushes: if node > 0 { leaf_brushes(node as usize) } else { Vec::new() },
            first_coll_aabb: u32_at(image, p),
            coll_aabb_count: u32_at(image, p + 4),
            mins: v3(image, p + 16),
            maxs: v3(image, p + 28),
        }
    };
    if let Some((at, n)) = c.nodes {
        for i in 0..n {
            let p = at + 16 * i;
            let pl = c.plane_index(u64_at(image, p)).map(plane).ok_or("a BSP node's plane could not be resolved")?;
            out.nodes.push(ClipNode { plane: pl, children: [u32_at(image, p + 8) as i32, u32_at(image, p + 12) as i32] });
        }
    }
    if let Some((at, n)) = c.leafs {
        for i in 0..n {
            out.leafs.push(leaf_at(at + CLIP_LEAF * i, 40));
        }
    }
    if let Some((at, n)) = c.verts {
        out.verts = (0..n).map(|i| v3(image, at + 12 * i)).collect();
    }
    if let Some((at, n)) = c.tri_indices {
        out.tris = (0..n).map(|i| std::array::from_fn(|k| u32_at(image, at + 12 * i + 4 * k))).collect();
    }
    if let Some((at, n)) = c.walkable {
        out.walkable = image[at..at + n].to_vec();
    }
    if let Some((at, n)) = c.partitions {
        out.partitions = (0..n).map(|i| ClipPartition { tri_count: image[at + 16 * i], first_tri: u32_at(image, at + 16 * i + 4) }).collect();
    }
    if let Some((at, n)) = c.aabb_trees {
        out.aabb_trees = (0..n)
            .map(|i| {
                let p = at + 32 * i;
                ClipAabb {
                    origin: v3(image, p),
                    material: u16_at(image, p + 12),
                    child_count: u16_at(image, p + 14),
                    half_size: v3(image, p + 16),
                    u: u32_at(image, p + 28),
                }
            })
            .collect();
    }
    for m in &c.static_models {
        let e = m.at;
        let inv = [v3(image, e + 32), v3(image, e + 44), v3(image, e + 56)];
        let len = (inv[0][0] * inv[0][0] + inv[0][1] * inv[0][1] + inv[0][2] * inv[0][2]).sqrt();
        let scale = if len > 1e-6 { 1.0 / len } else { 1.0 };
        out.static_models.push(ClipStaticModel {
            name: m.model.as_deref().map(bare).unwrap_or_default().to_owned(),
            contents: u32_at(image, e + 16),
            origin: v3(image, e + 20),
            axis: inv.map(|r| r.map(|x| x * scale)),
            scale,
        });
    }
    if let Some((at, n)) = c.cmodels {
        for i in 0..n {
            let p = at + CLIP_CMODEL * i;
            out.models.push(ClipModel { mins: v3(image, p), maxs: v3(image, p + 12), radius: f32_at(image, p + 24), leaf: leaf_at(p + 40, 40) });
        }
    }
    Ok(out)
}
