//! bo2mp: a Ghosts multiplayer map's world, collision and entities as plain
//! data, from what the exact walk kept (`walk::GfxWorldRec`, `ClipMapRec`,
//! `MapEntsRec`, `ComWorldRec`).
//!
//! Ghosts' world vertex is CoD4's layout (position, binormal sign, colour,
//! texture and lightmap coordinates, normal, tangent) with its unit vectors
//! packed unsigned 10:10:10 over 1023 (as its models'); surfaces index the
//! vertex table from `firstVertex` and the index table from `baseIndex`.
//! Collision is IW5's shape: brushes (bounds plus their non-axial sides),
//! terrain triangles under an AABB forest, brush models (`cmodel_t`) whose
//! brush trees are the leaf brush nodes. Spawn points are not in the entity
//! string but in `MapEnts::spawnList`; [`IwMap::entity_string`] writes them
//! back as entities so every consumer of IW entity text finds them.

use crate::layout as L;
use crate::walk::{AssetId, Capture, ClipMapRec, GfxWorldRec};

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldVertex {
    pub xyz: [f32; 3],
    pub binormal_sign: f32,
    /// D3DCOLOR bytes as stored (B, G, R, A).
    pub color: u32,
    pub tex_coord: [f32; 2],
    pub lmap_coord: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Surface {
    pub first_vertex: u32,
    pub vertex_count: u16,
    pub tri_count: u16,
    pub base_index: u32,
    pub material: Option<AssetId>,
    pub lightmap_index: u8,
    pub reflection_probe_index: u8,
    pub primary_light_env: u8,
    pub flags: u8,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StaticModel {
    pub model: Option<AssetId>,
    pub origin: [f32; 3],
    /// Rows: the model's forward, left and up axes in world space.
    pub axis: [[f32; 3]; 3],
    pub scale: f32,
    pub cull_dist: f32,
    pub flags: u16,
    /// Linear light where the model stands (`groundLighting`); all zero
    /// for a model lit per vertex.
    pub ground_lighting: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BrushModel {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub start_surf: u32,
    pub surface_count: u32,
}

/// The drawn world.
#[derive(Clone, Debug, Default)]
pub struct World {
    pub name: String,
    pub vertices: Vec<WorldVertex>,
    pub indices: Vec<u16>,
    pub surfaces: Vec<Surface>,
    /// Surface ranges [begin, end): lit opaque, lit decal, lit transparent,
    /// emissive.
    pub ranges: [(u32, u32); 4],
    pub static_models: Vec<StaticModel>,
    pub brush_models: Vec<BrushModel>,
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    /// The sky surfaces (every sky's `skyStartSurfs`).
    pub sky_surfaces: Vec<u32>,
    pub sky_images: Vec<Option<AssetId>>,
    pub lightmaps: Vec<(Option<AssetId>, Option<AssetId>)>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Plane {
    pub normal: [f32; 3],
    pub dist: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Brush {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub contents: i32,
    /// Its non-axial sides: plane index (into `Clip::planes`), material.
    pub sides: Vec<(u32, u16)>,
    /// Material per axial side: [min x, y, z], [max x, y, z].
    pub axial_materials: [[i16; 3]; 2],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AabbNode {
    pub origin: [f32; 3],
    pub half_size: [f32; 3],
    pub material_index: u16,
    pub child_count: u16,
    /// First child when `child_count > 0`, else a partition index.
    pub index: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CModel {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub radius: f32,
    pub first_coll_aabb: u16,
    pub coll_aabb_count: u16,
    /// Index into `Clip::leaf_brush_nodes` (-1: none, or its own ClipInfo).
    pub leaf_brush_node: i32,
}

#[derive(Clone, Debug, Default)]
pub struct LeafBrushNode {
    pub leaf_brush_count: i16,
    pub brushes: Vec<u16>,
    pub child_offsets: [u16; 2],
}

/// A static model's collision placement (`cStaticModel_s`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipStaticModel {
    pub model: Option<AssetId>,
    pub origin: [f32; 3],
    pub inv_scaled_axis: [[f32; 3]; 3],
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DynEnt {
    /// 1 clutter, 2 destructible (`DynEntityType`).
    pub kind: i32,
    pub quat: [f32; 4],
    pub origin: [f32; 3],
    pub model: Option<AssetId>,
    pub brush_model: u16,
    pub health: i32,
    pub contents: i32,
}

/// The collision.
#[derive(Clone, Debug, Default)]
pub struct Clip {
    pub name: String,
    pub planes: Vec<Plane>,
    /// (name, surface flags, contents).
    pub materials: Vec<(String, i32, i32)>,
    pub brushes: Vec<Brush>,
    pub verts: Vec<[f32; 3]>,
    pub tri_indices: Vec<u16>,
    /// (triangle count, first triangle, first vertex segment: a
    /// partition's triangles index the vertices from `segment * 1024`).
    pub partitions: Vec<(u8, i32, u8)>,
    pub aabb_trees: Vec<AabbNode>,
    pub cmodels: Vec<CModel>,
    pub leaf_brush_nodes: Vec<LeafBrushNode>,
    /// The static models' collision placements.
    pub static_models: Vec<ClipStaticModel>,
    pub dyn_ents: Vec<DynEnt>,
    /// Sides whose plane was not found (should be 0).
    pub missing_planes: usize,
}

/// A spawn point of `MapEnts::spawnList`.
#[derive(Clone, Debug, Default)]
pub struct SpawnRecord {
    pub classname: String,
    pub targetname: String,
    pub target: String,
    pub script_noteworthy: String,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
}

/// The sun (the ComWorld's first primary light when it is a sun).
#[derive(Clone, Copy, Debug, Default)]
pub struct Sun {
    /// Toward the sun.
    pub dir: [f32; 3],
    pub color: [f32; 3],
}

/// One hull of a trigger model: its box and the slabs that cut it.
#[derive(Clone, Debug, Default)]
pub struct TriggerHull {
    pub mid: [f32; 3],
    pub half: [f32; 3],
    /// (direction, mid point, half size).
    pub slabs: Vec<([f32; 3], f32, f32)>,
}

/// The entity keys Ghosts stores as numbers (its canonical script strings),
/// by name. Read off the retail maps' entities by their values (a key whose
/// values are spawn classes is `classname`, ...); keys not here are written
/// as `iw6_<number>`.
pub const IW6_KEYS: &[(u32, &str)] = &[
    (47, "angles"),
    (52, "animscript"),
    (97, "classname"),
    (272, "height"),
    (392, "model"),
    (430, "origin"),
    (476, "radius"),
    (504, "script_linkname"),
    (506, "script_noteworthy"),
    (542, "spawnflags"),
    (640, "target"),
    (642, "targetname"),
    (759, "sunlight"),
    (760, "suncolor"),
    (761, "sundirection"),
    (1619, "_color"),
    (27614, "script_gameobjectname"),
    (27658, "script_label"),
    (27665, "script_linkto"),
];

/// The name of a numbered entity key.
pub fn key_name(id: u32) -> Option<&'static str> {
    IW6_KEYS.iter().find(|(k, _)| *k == id).map(|(_, n)| *n)
}

/// Ghosts' entity text with its keys named (`97 "worldspawn"` ->
/// `"classname" "worldspawn"`) and its trigger models (`"model" "?N"`)
/// kept as they are (`?N` is the N-th of `IwMap::trigger_models`, as IW4's
/// and IW5's; `*N` the N-th brush model).
pub fn named_entity_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 3);
    for line in text.lines() {
        let t = line.trim();
        match t.split_once(' ') {
            Some((key, value)) if !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()) => {
                let id: u32 = key.parse().unwrap_or(u32::MAX);
                match key_name(id) {
                    Some(name) => out.push_str(&format!("\"{name}\" {}", value.trim())),
                    None => out.push_str(&format!("\"iw6_{id}\" {}", value.trim())),
                }
            }
            _ => out.push_str(t),
        }
        out.push('\n');
    }
    out
}

#[derive(Clone, Debug, Default)]
pub struct IwMap {
    pub world: Option<World>,
    pub clip: Option<Clip>,
    /// The trigger models (`"model" "?N"`), each its hulls.
    pub trigger_models: Vec<Vec<TriggerHull>>,
    /// The entity string as stored (numbered keys).
    pub entities: String,
    pub spawns: Vec<SpawnRecord>,
    pub sun: Option<Sun>,
    pub notes: Vec<String>,
}

impl IwMap {
    /// The entity string, keys named, with the spawn records written in as
    /// entities (`classname` the record's name: `mp_tdm_spawn`, ...).
    pub fn entity_string(&self) -> String {
        let mut out = named_entity_text(self.entities.trim_end_matches('\0'));
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        for s in &self.spawns {
            if s.classname.is_empty() {
                continue;
            }
            out.push_str("{\n");
            out.push_str(&format!("\"classname\" \"{}\"\n", s.classname));
            out.push_str(&format!("\"origin\" \"{} {} {}\"\n", s.origin[0], s.origin[1], s.origin[2]));
            out.push_str(&format!("\"angles\" \"{} {} {}\"\n", s.angles[0], s.angles[1], s.angles[2]));
            for (k, v) in [("targetname", &s.targetname), ("target", &s.target), ("script_noteworthy", &s.script_noteworthy)] {
                if !v.is_empty() {
                    out.push_str(&format!("\"{k}\" \"{v}\"\n"));
                }
            }
            out.push_str("}\n");
        }
        out
    }
}

struct R<'a>(&'a [u8]);

impl R<'_> {
    fn u8(&self, at: usize) -> u8 {
        self.0.get(at).copied().unwrap_or(0)
    }
    fn u16(&self, at: usize) -> u16 {
        self.0.get(at..at + 2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]))
    }
    fn i16(&self, at: usize) -> i16 {
        self.u16(at) as i16
    }
    fn u32(&self, at: usize) -> u32 {
        self.0.get(at..at + 4).map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn i32(&self, at: usize) -> i32 {
        self.u32(at) as i32
    }
    fn f32(&self, at: usize) -> f32 {
        f32::from_bits(self.u32(at))
    }
    fn vec3(&self, at: usize) -> [f32; 3] {
        [self.f32(at), self.f32(at + 4), self.f32(at + 8)]
    }
    /// `Bounds { midPoint, halfSize }` as (mins, maxs).
    fn bounds(&self, at: usize) -> ([f32; 3], [f32; 3]) {
        let (m, h) = (self.vec3(at), self.vec3(at + 12));
        (std::array::from_fn(|k| m[k] - h[k]), std::array::from_fn(|k| m[k] + h[k]))
    }
    fn fits(&self, at: usize, len: usize) -> bool {
        at.checked_add(len).is_some_and(|end| end <= self.0.len())
    }
}

/// Ghosts' packed unit vector (unsigned 10:10:10 over 1023).
pub fn unpack_unit_vec(packed: u32) -> [f32; 3] {
    std::array::from_fn(|k| ((packed >> (10 * k)) & 0x3ff) as f32 / 1023.0 * 2.0 - 1.0)
}

/// The map in `cap` (its first GfxWorld, ClipMap, MapEnts and ComWorld).
pub fn read_map(cap: &Capture, image: &[u8], script_strings: &[Option<String>]) -> IwMap {
    let r = R(image);
    let mut out = IwMap::default();
    if let Some(g) = cap.gfx_worlds.first() {
        match read_world(&r, g) {
            Ok(w) => out.world = Some(w),
            Err(e) => out.notes.push(format!("iw6 world: {e}")),
        }
    } else {
        out.notes.push("iw6 world: the zone has no GfxWorld".into());
    }
    if let Some(c) = cap.clip_maps.first() {
        let clip = read_clip(&r, c);
        if clip.missing_planes > 0 {
            out.notes.push(format!("iw6 clip: {} brush sides without their plane", clip.missing_planes));
        }
        out.clip = Some(clip);
    } else {
        out.notes.push("iw6 clip: the zone has no clipMap".into());
    }
    if let Some(m) = cap.map_ents.first() {
        out.entities = m.entity_string.clone();
        out.trigger_models = read_triggers(&r, m.triggers);
        let sz = L::SpawnPointEntityRecord::SIZE;
        if let Some(span) = m.spawns {
            let string = |at: usize| -> String {
                let i = r.u32(at) as usize;
                script_strings.get(i).cloned().flatten().unwrap_or_default()
            };
            for i in 0..span.len / sz {
                let s = span.pos + sz * i;
                use L::SpawnPointEntityRecord as P;
                out.spawns.push(SpawnRecord {
                    classname: string(s + P::name),
                    targetname: String::new(),
                    target: string(s + P::target),
                    script_noteworthy: string(s + P::script_noteworthy),
                    origin: r.vec3(s + P::origin),
                    angles: r.vec3(s + P::angles),
                });
            }
        }
    } else {
        out.notes.push("iw6 entities: the zone has no MapEnts".into());
    }
    if let Some(cw) = cap.com_worlds.first()
        && let Some(lights) = cw.primary_lights
        && cw.primary_light_count > 0
    {
        // Light 0 is the "no light" slot; the sun is the first of type 1.
        let pl = L::ComPrimaryLight::SIZE;
        for i in 0..cw.primary_light_count as usize {
            let l = lights.pos + pl * i;
            if r.u8(l + L::ComPrimaryLight::r#type) == 1 {
                let dir = r.vec3(l + L::ComPrimaryLight::dir);
                out.sun = Some(Sun { dir, color: r.vec3(l + L::ComPrimaryLight::color) });
                break;
            }
        }
    }
    out
}

fn read_world(r: &R, g: &GfxWorldRec) -> Result<World, String> {
    use L::GfxWorld as G;
    let h = g.header;
    let mut w = World { name: g.name.clone(), ..Default::default() };
    let vs = L::GfxWorldVertex::SIZE;
    let vat = g.vertices.ok_or("no vertices in the file")?;
    let vn = g.vertex_count as usize;
    if !r.fits(vat, vs * vn) {
        return Err(format!("{vn} vertices run past the zone"));
    }
    w.vertices = (0..vn)
        .map(|i| {
            let v = vat + vs * i;
            use L::GfxWorldVertex as V;
            WorldVertex {
                xyz: r.vec3(v + V::xyz),
                binormal_sign: r.f32(v + V::binormalSign),
                color: r.u32(v + V::color),
                tex_coord: [r.f32(v + V::texCoord), r.f32(v + V::texCoord + 4)],
                lmap_coord: [r.f32(v + V::lmapCoord), r.f32(v + V::lmapCoord + 4)],
                normal: unpack_unit_vec(r.u32(v + V::normal)),
                tangent: unpack_unit_vec(r.u32(v + V::tangent)),
            }
        })
        .collect();
    let iat = g.indices.ok_or("no indices in the file")?;
    let n = g.index_count as usize;
    if !r.fits(iat, 2 * n) {
        return Err(format!("{n} indices run past the zone"));
    }
    w.indices = (0..n).map(|i| r.u16(iat + 2 * i)).collect();
    let sat = g.surfaces.ok_or("no surfaces in the file")?;
    let gs = L::GfxSurface::SIZE;
    let sb = L::GfxSurfaceBounds::SIZE;
    for i in 0..g.surface_count as usize {
        let s = sat + gs * i;
        use L::GfxSurface as S;
        let (mins, maxs) = g.surface_bounds.map_or(([0.0; 3], [0.0; 3]), |b| r.bounds(b + sb * i));
        w.surfaces.push(Surface {
            first_vertex: r.u32(s + S::tris__firstVertex),
            vertex_count: r.u16(s + S::tris__vertexCount),
            tri_count: r.u16(s + S::tris__triCount),
            base_index: r.u32(s + S::tris__baseIndex),
            material: g.surface_materials.get(i).copied().flatten(),
            lightmap_index: r.u8(s + S::laf__fields__lightmapIndex),
            reflection_probe_index: r.u8(s + S::laf__fields__reflectionProbeIndex),
            primary_light_env: r.u8(s + S::laf__fields__primaryLightEnvIndex),
            flags: r.u8(s + S::laf__fields__flags),
            mins,
            maxs,
        });
    }
    let range = |b: usize, e: usize| (r.u32(h + b), r.u32(h + e));
    w.ranges = [
        range(G::dpvs__litOpaqueSurfsBegin, G::dpvs__litOpaqueSurfsEnd),
        range(G::dpvs__litDecalSurfsBegin, G::dpvs__litDecalSurfsEnd),
        range(G::dpvs__litTransSurfsBegin, G::dpvs__litTransSurfsEnd),
        range(G::dpvs__emissiveSurfsBegin, G::dpvs__emissiveSurfsEnd),
    ];
    if let Some(at) = g.smodels {
        let sd = L::GfxStaticModelDrawInst::SIZE;
        for i in 0..g.smodel_count as usize {
            let d = at + sd * i;
            use L::GfxStaticModelDrawInst as D;
            let a = d + D::placement__axis;
            w.static_models.push(StaticModel {
                model: g.smodel_models.get(i).copied().flatten(),
                origin: r.vec3(d + D::placement__origin),
                axis: [r.vec3(a), r.vec3(a + 12), r.vec3(a + 24)],
                scale: r.f32(d + D::placement__scale),
                cull_dist: f32::from(r.u16(d + D::cullDist)),
                flags: r.u16(d + D::flags),
                ground_lighting: std::array::from_fn(|k| r.f32(d + D::groundLighting + 4 * k)),
            });
        }
    }
    if let Some(at) = g.models {
        let bm = L::GfxBrushModel::SIZE;
        for i in 0..g.model_count as usize {
            let m = at + bm * i;
            let (mins, maxs) = r.bounds(m + L::GfxBrushModel::bounds);
            w.brush_models.push(BrushModel {
                mins,
                maxs,
                start_surf: r.u32(m + L::GfxBrushModel::startSurfIndex),
                surface_count: r.u32(m + L::GfxBrushModel::surfaceCount),
            });
        }
    }
    (w.mins, w.maxs) = r.bounds(h + G::bounds);
    for s in g.sky_surfaces.iter().flatten() {
        for k in 0..s.len / 4 {
            w.sky_surfaces.push(r.u32(s.pos + 4 * k));
        }
    }
    w.sky_images = g.sky_images.clone();
    w.lightmaps = g.lightmaps.clone();
    Ok(w)
}

fn read_clip(r: &R, c: &ClipMapRec) -> Clip {
    let info = &c.info;
    let mut clip = Clip { name: c.name.clone(), materials: info.materials.clone(), ..Default::default() };
    // Planes: the info's array, then any a side names outside it.
    let ps = L::cplane_s::SIZE;
    let read_plane = |at: usize| Plane { normal: r.vec3(at), dist: r.f32(at + 12) };
    if let Some(at) = info.planes {
        clip.planes = (0..info.plane_count as usize).map(|i| read_plane(at + ps * i)).collect();
    }
    let mut extra: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
    let mut plane_index = |clip: &mut Clip, at: usize| -> u32 {
        if let Some(base) = info.planes {
            let end = base + ps * info.plane_count as usize;
            if (base..end).contains(&at) && (at - base).is_multiple_of(ps) {
                return ((at - base) / ps) as u32;
            }
        }
        *extra.entry(at).or_insert_with(|| {
            clip.planes.push(read_plane(at));
            (clip.planes.len() - 1) as u32
        })
    };
    let bs = L::cbrushside_t::SIZE;
    let b = L::cbrush_t::SIZE;
    if let Some(at) = info.brushes {
        for i in 0..info.brush_count as usize {
            let br = at + b * i;
            let numsides = r.u16(br + L::cbrush_t::numsides) as usize;
            let (mins, maxs) = info.brush_bounds.map_or(([0.0; 3], [0.0; 3]), |bb| r.bounds(bb + 24 * i));
            let contents = info.brush_contents.map_or(0, |bc| r.i32(bc + 4 * i));
            let am = br + L::cbrush_t::axialMaterialNum;
            let axial_materials = [
                [r.i16(am), r.i16(am + 2), r.i16(am + 4)],
                [r.i16(am + 6), r.i16(am + 8), r.i16(am + 10)],
            ];
            let mut sides = Vec::with_capacity(numsides);
            if let Some(first) = info.brush_sides.get(i).copied().flatten() {
                for k in 0..numsides {
                    let side = first + bs * k;
                    let material = r.u16(side + L::cbrushside_t::materialNum);
                    match info.side_planes.get(&side) {
                        Some(&p) => {
                            let idx = plane_index(&mut clip, p);
                            sides.push((idx, material));
                        }
                        None => clip.missing_planes += 1,
                    }
                }
            } else if numsides > 0 {
                clip.missing_planes += numsides;
            }
            clip.brushes.push(Brush { mins, maxs, contents, sides, axial_materials });
        }
    }
    if let Some(at) = info.leaf_brush_nodes {
        let ln = L::cLeafBrushNode_s::SIZE;
        for i in 0..info.leaf_brush_node_count as usize {
            let node = at + ln * i;
            use L::cLeafBrushNode_s as N;
            let count = r.i16(node + N::leafBrushCount);
            let brushes = match info.leaf_brush_lists.get(i).copied().flatten() {
                Some(list) if count > 0 => (0..count as usize).map(|k| r.u16(list + 2 * k)).collect(),
                _ => Vec::new(),
            };
            let child_offsets = if count > 0 {
                [0, 0]
            } else {
                [r.u16(node + N::data__children__childOffset), r.u16(node + N::data__children__childOffset + 2)]
            };
            clip.leaf_brush_nodes.push(LeafBrushNode { leaf_brush_count: count, brushes, child_offsets });
        }
    }
    if let Some(at) = c.verts {
        clip.verts = (0..c.vert_count as usize).map(|i| r.vec3(at + 12 * i)).collect();
    }
    if let Some(at) = c.tri_indices {
        clip.tri_indices = (0..3 * c.tri_count as usize).map(|i| r.u16(at + 2 * i)).collect();
    }
    if let Some(at) = c.partitions {
        let cp = L::CollisionPartition::SIZE;
        clip.partitions = (0..c.partition_count as usize)
            .map(|i| {
                let p = at + cp * i;
                (
                    r.u8(p + L::CollisionPartition::triCount),
                    r.i32(p + L::CollisionPartition::firstTri),
                    r.u8(p + L::CollisionPartition::firstVertSegment),
                )
            })
            .collect();
    }
    if let Some(at) = c.aabb_trees {
        let ab = L::CollisionAabbTree::SIZE;
        use L::CollisionAabbTree as A;
        clip.aabb_trees = (0..c.aabb_tree_count as usize)
            .map(|i| {
                let t = at + ab * i;
                AabbNode {
                    origin: r.vec3(t + A::midPoint),
                    half_size: r.vec3(t + A::halfSize),
                    material_index: r.u16(t + A::materialIndex),
                    child_count: r.u16(t + A::childCount),
                    index: r.i32(t + A::u),
                }
            })
            .collect();
    }
    if let Some(at) = c.cmodels {
        let cm = L::cmodel_t::SIZE;
        use L::cmodel_t as M;
        for i in 0..c.cmodel_count as usize {
            let m = at + cm * i;
            let (mins, maxs) = r.bounds(m + M::bounds);
            let own = c.cmodel_own_info.get(i).copied().unwrap_or(false);
            clip.cmodels.push(CModel {
                mins,
                maxs,
                radius: r.f32(m + M::radius),
                first_coll_aabb: r.u16(m + M::leaf__firstCollAabbIndex),
                coll_aabb_count: r.u16(m + M::leaf__collAabbCount),
                leaf_brush_node: if own { -1 } else { r.i32(m + M::leaf__leafBrushNode) },
            });
        }
    }
    if let Some(at) = c.static_models {
        let sm = L::cStaticModel_s::SIZE;
        use L::cStaticModel_s as S;
        for i in 0..c.static_model_models.len() {
            let s = at + sm * i;
            let a = s + S::invScaledAxis;
            let (mins, maxs) = r.bounds(s + S::absBounds);
            clip.static_models.push(ClipStaticModel {
                model: c.static_model_models[i],
                origin: r.vec3(s + S::origin),
                inv_scaled_axis: [r.vec3(a), r.vec3(a + 12), r.vec3(a + 24)],
                mins,
                maxs,
            });
        }
    }
    for k in 0..2 {
        let Some(at) = c.dyn_ent_defs[k] else { continue };
        let de = L::DynEntityDef::SIZE;
        use L::DynEntityDef as D;
        for i in 0..c.dyn_ent_counts[k] as usize {
            let d = at + de * i;
            clip.dyn_ents.push(DynEnt {
                kind: r.i32(d + D::r#type),
                quat: [r.f32(d + D::pose__quat), r.f32(d + D::pose__quat + 4), r.f32(d + D::pose__quat + 8), r.f32(d + D::pose__quat + 12)],
                origin: r.vec3(d + D::pose__origin),
                model: c.dyn_ent_models[k].get(i).copied().flatten(),
                brush_model: r.u16(d + D::brushModel),
                health: i32::from(r.i16(d + D::health)),
                contents: r.i32(d + D::contents),
            });
        }
    }
    clip
}

fn read_triggers(r: &R, [models, hulls, slabs]: [Option<crate::walk::Span>; 3]) -> Vec<Vec<TriggerHull>> {
    let (Some(models), Some(hulls)) = (models, hulls) else {
        return Vec::new();
    };
    let hull_count = hulls.len / L::TriggerHull::SIZE;
    let slab_count = slabs.map_or(0, |s| s.len / L::TriggerSlab::SIZE);
    (0..models.len / L::TriggerModel::SIZE)
        .map(|i| {
            let m = models.pos + L::TriggerModel::SIZE * i;
            let count = usize::from(r.u16(m + L::TriggerModel::hullCount));
            let first = usize::from(r.u16(m + L::TriggerModel::firstHull));
            if first + count > hull_count {
                return Vec::new();
            }
            (first..first + count)
                .map(|n| {
                    let h = hulls.pos + L::TriggerHull::SIZE * n;
                    let sc = usize::from(r.u16(h + L::TriggerHull::slabCount));
                    let sf = usize::from(r.u16(h + L::TriggerHull::firstSlab));
                    let slabs = match slabs {
                        Some(sp) if sf + sc <= slab_count => (sf..sf + sc)
                            .map(|k| {
                                let s = sp.pos + L::TriggerSlab::SIZE * k;
                                (r.vec3(s + L::TriggerSlab::dir), r.f32(s + L::TriggerSlab::midPoint), r.f32(s + L::TriggerSlab::halfSize))
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    TriggerHull {
                        mid: r.vec3(h + L::TriggerHull::bounds__midPoint),
                        half: r.vec3(h + L::TriggerHull::bounds__halfSize),
                        slabs,
                    }
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn keys_are_named() {
        let t = "{\n97 \"worldspawn\"\n430 \"0 0 0\"\n99999 \"x\"\n}";
        let n = super::named_entity_text(t);
        assert!(n.contains("\"classname\" \"worldspawn\""), "{n}");
        assert!(n.contains("\"origin\" \"0 0 0\""), "{n}");
        assert!(n.contains("\"iw6_99999\" \"x\""), "{n}");
    }
}

/// A raw file of the zone by name (case-insensitive), inflated when stored
/// compressed.
pub fn raw_file(cap: &Capture, image: &[u8], name: &str) -> Option<Vec<u8>> {
    let r = cap.raw_files.iter().find(|r| r.name.eq_ignore_ascii_case(name))?;
    let data = r.data?.get(image);
    if r.compressed_len == 0 {
        return Some(data.to_vec());
    }
    use std::io::Read;
    let mut out = Vec::with_capacity(r.len as usize);
    flate2::read::ZlibDecoder::new(data).read_to_end(&mut out).ok()?;
    Some(out)
}

/// An IEEE half float.
pub fn half_to_f32(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = (h >> 10) & 0x1f;
    let frac = f32::from(h & 0x3ff);
    sign * match exp {
        0 => frac * 2f32.powi(-24),
        31 => f32::INFINITY,
        e => (1.0 + frac / 1024.0) * 2f32.powi(i32::from(e) - 15),
    }
}

/// One lightmap page in Call of Duty 4's form (what the fallback draw's
/// IW3 branch reads): the primary as 8-bit luminance (`D3DFMT_L8`) and the
/// secondary's two stacked pages as `A8R8G8B8` (bytes B, G, R, A).
#[derive(Clone, Debug)]
pub struct Iw3Lightmap {
    pub primary: (u16, u16, Vec<u8>),
    pub secondary: (u16, u16, Vec<u8>),
}

/// Ghosts' lightmap pair as Call of Duty 4's.
///
/// Ghosts' primary is BC4 at twice the secondary page's size (the sun's
/// baked visibility, as CoD4's L8 primary); its secondary is two pages
/// stacked (`R16G16B16A16_FLOAT`), the upper the softer, as CoD4's A and B:
/// linear light, alpha a tangent direction in -1..1. Here the light becomes
/// CoD4's gamma bytes (`sqrt`, clamped at 1), the direction CoD4's alpha
/// code (`a * 4.08 - 2.08`), and the primary is box-filtered down to the
/// page's size (the full size made a 2048 x 6144 page per lightmap).
pub fn iw3_lightmap(
    cap: &Capture,
    image: &[u8],
    pair: (Option<AssetId>, Option<AssetId>),
) -> Result<Iw3Lightmap, String> {
    let get = |id: Option<AssetId>| -> Result<&crate::walk::ImageRec, String> {
        id.and_then(|i| cap.images.get(i.index as usize)).ok_or_else(|| "image not in the zone".to_owned())
    };
    let (p, s) = (get(pair.0)?, get(pair.1)?);
    let mut paks = crate::image::ImagePaks::new(std::path::Path::new("."));
    let pp = crate::image::image_pixels(p, image, &mut paks, 1 << 14)?;
    let sp = crate::image::image_pixels(s, image, &mut paks, 1 << 14)?;
    if !matches!(pp.dxgi_format, 79 | 80) {
        return Err(format!("primary format {} (BC4 expected)", pp.dxgi_format));
    }
    if sp.dxgi_format != 10 {
        return Err(format!("secondary format {} (R16G16B16A16_FLOAT expected)", sp.dxgi_format));
    }
    let (sw, sh) = (sp.width as usize, sp.height as usize);
    if sw == 0 || sh < 2 || sh % 2 != 0 {
        return Err(format!("secondary {sw}x{sh}"));
    }
    let need = sw * sh * 8;
    if sp.data.len() < need {
        return Err(format!("secondary has {} of {need} bytes", sp.data.len()));
    }
    let mut secondary = vec![0u8; sw * sh * 4];
    for i in 0..sw * sh {
        let h = |c: usize| half_to_f32(u16::from_le_bytes([sp.data[i * 8 + 2 * c], sp.data[i * 8 + 2 * c + 1]]));
        let g = |v: f32| (v.clamp(0.0, 1.0).sqrt() * 255.0).round() as u8;
        let a = (((h(3).clamp(-1.0, 1.0) + 2.08) / 4.08) * 255.0).round().clamp(0.0, 255.0) as u8;
        secondary[i * 4..i * 4 + 4].copy_from_slice(&[g(h(2)), g(h(1)), g(h(0)), a]);
    }
    // The primary, BC4 decoded.
    let (pw, ph) = (pp.width as usize, pp.height as usize);
    if pw % 4 != 0 || ph % 4 != 0 {
        return Err(format!("primary {pw}x{ph} is not whole blocks"));
    }
    let blocks = (pw / 4) * (ph / 4);
    if pp.data.len() < blocks * 8 {
        return Err(format!("primary has {} of {} bytes", pp.data.len(), blocks * 8));
    }
    let mut full = vec![0u8; pw * ph];
    for by in 0..ph / 4 {
        for bx in 0..pw / 4 {
            let b = (by * (pw / 4) + bx) * 8;
            let at = (by * 4) * pw + bx * 4;
            bcdec_rs::bc4(&pp.data[b..b + 8], &mut full[at..], pw, pp.dxgi_format == 81);
        }
    }
    // Down to the page's size (whole steps only).
    let page_h = sh / 2;
    let (fx, fy) = (pw / sw.max(1), ph / page_h.max(1));
    let (fx, fy) = if fx >= 1 && fy >= 1 && pw == sw * fx && ph == page_h * fy { (fx, fy) } else { (1, 1) };
    let (ow, oh) = (pw / fx, ph / fy);
    let mut primary = vec![0u8; ow * oh];
    for y in 0..oh {
        for x in 0..ow {
            let mut sum = 0u32;
            for dy in 0..fy {
                for dx in 0..fx {
                    sum += u32::from(full[(y * fy + dy) * pw + x * fx + dx]);
                }
            }
            primary[y * ow + x] = (sum / (fx * fy) as u32) as u8;
        }
    }
    let to16 = |v: usize| u16::try_from(v).map_err(|_| format!("{v} is too large"));
    Ok(Iw3Lightmap { primary: (to16(ow)?, to16(oh)?, primary), secondary: (to16(sw)?, to16(sh)?, secondary) })
}

