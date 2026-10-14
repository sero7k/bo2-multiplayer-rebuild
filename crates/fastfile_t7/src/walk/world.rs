//! The map assets: `MapEnts`, `ComWorld`, `GameWorld`, `ClipMap` (and the
//! small map-only types around them). Layouts from x64-zt's T7 structs
//! (their x64 sizes match the header sizes the zones load), stream order
//! read on retail map zones (`mp_nuketown_x`, ...): each pointer field in
//! offset order, an array then what its records point to.
//!
//! What a converter needs is kept in `WorldCapture` (file offsets of the
//! arrays, counts, and the names of the assets the records name).

use super::gfxworld::GfxWorldRec;
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::Stream;

/// A loaded array: (file offset, count).
pub type Span = Option<(usize, usize)>;

/// `MapEnts`: the entity string and the trigger tables.
#[derive(Clone, Debug, Default)]
pub struct MapEntsRec {
    pub name: String,
    pub entity_string: Option<(usize, usize)>,
    pub trigger_models: Span,
    pub trigger_hulls: Span,
    pub trigger_slabs: Span,
}

/// A static model the collision places (`cStaticModel_t`, 0x70).
#[derive(Clone, Debug, Default)]
pub struct ClipStaticModel {
    pub model: Option<String>,
    pub at: usize,
}

/// A pointer the clip map's records hold: data loaded right there (its
/// file offset) or a reference (its VIRTUAL offset, resolved at the end of
/// the clip map, `resolve_clip`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipPtr {
    Null,
    Here(usize),
    Ref(u64),
}

/// `clipMap_t`'s arrays (file offsets, counts) and what its records name.
#[derive(Clone, Debug, Default)]
pub struct ClipMapRec {
    pub name: String,
    pub header_at: usize,
    /// The planes (20 each); on retail a reference to the `GfxWorld`'s DPVS
    /// planes. `planes_virt`: the reference, which plane references index from.
    pub planes: Span,
    pub planes_virt: Option<u64>,
    /// (name, surface flags, content flags).
    pub materials: Vec<(String, i32, i32)>,
    pub brush_sides: Span,
    pub leaf_brush_nodes: Span,
    /// Per leaf brush node: its brush index list (file offset, count).
    pub leaf_brush_lists: Vec<Option<(usize, usize)>>,
    pub brushes: Span,
    pub brush_verts: Span,
    /// Per brush: its sides' and vertices' file offsets.
    pub brush_side_lists: Vec<Option<usize>>,
    pub brush_vert_lists: Vec<Option<usize>>,
    pub nodes: Span,
    pub leafs: Span,
    pub verts: Span,
    pub tri_indices: Span,
    pub partitions: Span,
    pub aabb_trees: Span,
    pub cmodels: Span,
    pub static_models: Vec<ClipStaticModel>,
    pub static_models_at: Option<usize>,
    pub dyn_ent_defs: [Span; 2],
    pub dyn_ent_models: Vec<Option<String>>,
    /// What our VIRTUAL count and the zone's differ by inside the clip map
    /// (measured from the brushes' first side and vertex references).
    pub virt_delta: i64,
    raw_side_lists: Vec<ClipPtr>,
    raw_vert_lists: Vec<ClipPtr>,
    raw_leaf_lists: Vec<(ClipPtr, usize)>,
}

impl ClipMapRec {
    /// A plane's index from a plane pointer's raw value (a reference into
    /// `planes`).
    pub fn plane_index(&self, raw: u64) -> Option<usize> {
        let crate::ptr::Ptr::Ref { offset, .. } = crate::ptr::Ptr::decode(raw) else { return None };
        let d = offset.checked_sub(self.planes_virt?)?;
        (d % 20 == 0).then_some((d / 20) as usize).filter(|&i| self.planes.is_some_and(|(_, n)| i < n))
    }}

/// `ComWorld`: the primary lights.
#[derive(Clone, Debug, Default)]
pub struct ComWorldRec {
    pub name: String,
    pub lights: Span,
}

/// `GameWorld`: the path nodes.
#[derive(Clone, Debug, Default)]
pub struct GameWorldRec {
    pub name: String,
    /// The placed nodes (`nodeCount`; the array has room for 128 more).
    pub nodes: Span,
    /// Per node: its links (16 each).
    pub links: Vec<Span>,
}

/// What the walk keeps of a map zone's world assets.
#[derive(Clone, Debug, Default)]
pub struct WorldCapture {
    pub map_ents: Vec<MapEntsRec>,
    pub com_worlds: Vec<ComWorldRec>,
    pub game_worlds: Vec<GameWorldRec>,
    pub clip_maps: Vec<ClipMapRec>,
    pub gfx_worlds: Vec<GfxWorldRec>,
}

fn count(s: &Stream, at: usize) -> Result<usize> {
    Ok(s.i32_at(at)?.max(0) as usize)
}

/// `records` that keeps (file offset, count).
#[track_caller]
pub(crate) fn span(s: &mut Stream, field: usize, n: usize, size: usize, mask: u64) -> Result<Span> {
    Ok(s.records(field, n, size, mask)?.map(|at| (at, n)))
}

/// `MapEnts` (0x48): `{ char* name; char* entityString; i32 numEntityChars;
/// MapTriggers { u32 count; TriggerModel* models (8); u32 hullCount;
/// TriggerHull* hulls (32); u32 slabCount; TriggerSlab* slabs (20) } }`.
/// Order: name, entity string (`numEntityChars` bytes), models, hulls, slabs.
pub fn map_ents(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::MapEnts.header_size(), &[0, 8, 0x20, 0x30, 0x40])?;
    let name = s.string(h)?;
    let chars = count(s, h + 0x10)?;
    let entity_string = s.array(h + 8, chars, 0)?.map(|at| (at, chars));
    let n = count(s, h + 0x18)?;
    let trigger_models = span(s, h + 0x20, n, 8, 3)?;
    let n = count(s, h + 0x28)?;
    let trigger_hulls = span(s, h + 0x30, n, 32, 3)?;
    let n = count(s, h + 0x38)?;
    let trigger_slabs = span(s, h + 0x40, n, 20, 3)?;
    s.capture.world.map_ents.push(MapEntsRec {
        name: name.clone().unwrap_or_default(),
        entity_string,
        trigger_models,
        trigger_hulls,
        trigger_slabs,
    });
    Ok(name)
}

/// `ComPrimaryLight` (564: x64-zt's `GfxConfig_Light` and `cookieIndex`;
/// the light def name at 400, on `mp_nuketown_x`).
pub const PRIMARY_LIGHT: usize = 564;
const UMBRA_TOME: usize = 0x50;

/// `ComWorld` (0x88 on retail: x64-zt's 0x68 and 32 more bytes, zero on
/// the maps read): `{ name; isInUse; u32 primaryLightCount; lights;
/// u32 lightExploderCount; ComLightExploder* (16: { hash; u32 n;
/// triggers (20 each) }); u32 sunVolumeCount; u32* sunVolumeProbeCount;
/// u32 probeCount; u32* probeGuids; u32 probeExploderCount;
/// ComProbeExploder* (16: { hash; u32 n; triggers (28 each) });
/// u32 umbraTomeCount; ComUmbraTome* (0x50: x64-zt's 0x40 and 16 zero
/// bytes) }`, loaded in that order.
pub fn com_world(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::ComWorld.header_size(), &[0, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60])?;
    let name = s.string(h)?;
    let n = count(s, h + 0xc)?;
    let lights = span(s, h + 0x10, n, PRIMARY_LIGHT, 3)?;
    let n = count(s, h + 0x18)?;
    if let Some(t) = s.records(h + 0x20, n, 16, 7)? {
        for i in 0..n {
            let k = count(s, t + 16 * i + 4)?;
            s.records(t + 16 * i + 8, k, 20, 3)?;
        }
    }
    let n = count(s, h + 0x28)?;
    s.records(h + 0x30, n, 4, 3)?;
    let n = count(s, h + 0x38)?;
    s.records(h + 0x40, n, 4, 3)?;
    let n = count(s, h + 0x48)?;
    if let Some(t) = s.records(h + 0x50, n, 16, 7)? {
        for i in 0..n {
            let k = count(s, t + 16 * i + 4)?;
            s.records(t + 16 * i + 8, k, 28, 3)?;
        }
    }
    let n = count(s, h + 0x58)?;
    if let Some(t) = s.records(h + 0x60, n, UMBRA_TOME, 7)? {
        for i in 0..n {
            let u = t + UMBRA_TOME * i;
            s.no_loads(u, UMBRA_TOME, &[0, 0x10, 0x20, 0x28, 0x38])?;
            s.string(u)?;
            let v = count(s, u + 8)?;
            s.records(u + 0x10, v, 24, 3)?;
            let k = count(s, u + 0x18)?;
            s.records(u + 0x20, k, 4, 3)?;
            s.records(u + 0x28, k, 4, 3)?;
            let g = count(s, u + 0x30)?;
            if let Some(gt) = s.records(u + 0x38, g, 16, 7)? {
                for j in 0..g {
                    s.string(gt + 16 * j)?;
                }
            }
        }
    }
    s.capture.world.com_worlds.push(ComWorldRec { name: name.clone().unwrap_or_default(), lights });
    Ok(name)
}

/// `pathnode_t` (224 on retail: x64-zt's constant part, 120, then the
/// dynamic and transient parts).
pub const PATH_NODE: usize = 224;
/// Nodes the game can add at runtime: the node array has room for them
/// (zeroed in the zone), as Black Ops II's.
pub const DYNAMIC_PATH_NODES: usize = 128;

/// `GameWorld` (0x50): `{ name; PathData { u32 nodeCount,
/// originalNodeCount; pathnode_t* nodes; basenodes; i32 visBytes; u8*
/// pathVis; i32 smoothBytes; u8* smoothCache; i32 nodeTreeCount;
/// pathnode_tree_t* nodeTree (24) } }`. Order read on `mp_nuketown_x`:
/// the name (a reference), the nodes (`nodeCount + 128`), each node's links
/// (16 each, count at 0x50), the base nodes (runtime: no file bytes), the
/// vis bytes, the smoothing cache, the node tree (its children are
/// references into itself), then each leaf's node indices (u16).
pub fn game_world(s: &mut Stream, h: usize) -> Result<Option<String>> {
    s.no_loads(h, AssetType::GameWorld.header_size(), &[0, 0x10, 0x18, 0x28, 0x38, 0x48])?;
    let name = s.string(h)?;
    let n = count(s, h + 8)?;
    let nodes = s.records(h + 0x10, n + DYNAMIC_PATH_NODES, PATH_NODE, 7)?;
    let mut links = Vec::new();
    if let Some(t) = nodes {
        for i in 0..n + DYNAMIC_PATH_NODES {
            let e = t + PATH_NODE * i;
            let k = s.u16_at(e + 0x50)? as usize;
            let at = s.records(e + 0x58, k, 16, 3)?;
            if i < n {
                links.push(at.map(|at| (at, k)));
            }
        }
    }
    s.push(crate::stream::RUNTIME_VIRTUAL);
    let r = s.records(h + 0x18, n + DYNAMIC_PATH_NODES, 16, 15);
    s.pop();
    r?;
    let k = count(s, h + 0x20)?;
    s.array(h + 0x28, k, 0)?;
    let k = count(s, h + 0x30)?;
    s.array(h + 0x38, k, 0)?;
    let trees = count(s, h + 0x40)?;
    if let Some(t) = s.records(h + 0x48, trees, 24, 7)? {
        for i in 0..trees {
            let e = t + 24 * i;
            if s.i32_at(e)? < 0 {
                let k = count(s, e + 8)?;
                s.records(e + 16, k, 2, 1)?;
            } else {
                s.no_loads(e + 8, 16, &[])?;
            }
        }
    }
    s.capture.world.game_worlds.push(GameWorldRec {
        name: name.clone().unwrap_or_default(),
        nodes: nodes.map(|at| (at, n)),
        links,
    });
    Ok(name)
}

/// A Havok-backed asset (`NavMesh` 0x68, `NavVolume` 0x48): `{ name;
/// then (u64 size; u8* data) pairs }`, each data a Havok packfile
/// (`hk_2014.2.0-r1`), loaded in order after the name.
pub fn havok_blobs(s: &mut Stream, h: usize, ty: AssetType) -> Result<Option<String>> {
    let size = ty.header_size();
    let fields: Vec<usize> = (0..(size - 8) / 16).map(|i| 0x10 + 16 * i).collect();
    let mut skip = fields.clone();
    skip.push(0);
    s.no_loads(h, size, &skip)?;
    let name = s.string(h)?;
    for f in fields {
        let n = s.u64_at(f + h - 8)? as usize;
        s.array(h + f, n, 15)?;
    }
    Ok(name)
}

pub fn nav_mesh(s: &mut Stream, h: usize) -> Result<Option<String>> {
    havok_blobs(s, h, AssetType::NavMesh)
}

pub fn nav_volume(s: &mut Stream, h: usize) -> Result<Option<String>> {
    havok_blobs(s, h, AssetType::NavVolume)
}

/// The file offset of the data a pointer field names: what follows (the
/// offset `records` returned) or, for a reference, where the walk read
/// that VIRTUAL offset.
pub(crate) fn target(s: &Stream, field: usize, loaded: Option<usize>) -> Result<Option<usize>> {
    Ok(match s.ptr_at(field)? {
        crate::ptr::Ptr::Ref { offset, .. } => s.vmap.file_pos(offset),
        _ => loaded,
    })
}

pub const CLIP_BRUSH: usize = 112;
pub const CLIP_STATIC_MODEL: usize = 112;
pub const CLIP_CMODEL: usize = 96;
pub const CLIP_LEAF: usize = 56;
pub const LEAF_BRUSH_NODE: usize = 24;
pub const DYN_ENT_DEF: usize = 152;
pub const PHYS_CONSTRAINT: usize = 208;

fn clip_ptr(s: &Stream, field: usize, loaded: Option<usize>) -> Result<ClipPtr> {
    Ok(match s.ptr_at(field)? {
        crate::ptr::Ptr::Ref { offset, .. } => ClipPtr::Ref(offset),
        _ => loaded.map_or(ClipPtr::Null, ClipPtr::Here),
    })
}

/// `ClipInfo` (0x90) at `c`: planes (20; on retail a reference to the
/// GfxWorld's), materials (16, then their names), brush sides (16: plane
/// references), leaf brush nodes (24, then each leaf's brush indices),
/// leaf brushes (u16), brush vertices (12), `uinds` (u32), brushes (112:
/// x64-zt's `cbrush_t` and 8 more bytes; sides and vertices are references
/// into the arrays above). Bounds and contents are runtime.
fn clip_info(s: &mut Stream, c: usize, rec: &mut ClipMapRec) -> Result<()> {
    let n = count(s, c)?;
    let planes = s.records(c + 8, n, 20, 3)?;
    rec.planes = target(s, c + 8, planes)?.map(|at| (at, n));
    rec.planes_virt = match s.ptr_at(c + 8)? {
        crate::ptr::Ptr::Ref { offset, .. } => Some(offset),
        _ => planes.and_then(|at| s.vmap.virt_of(at)),
    };
    let n = count(s, c + 0x10)?;
    if let Some(t) = s.records(c + 0x18, n, 16, 7)? {
        for i in 0..n {
            let m = t + 16 * i;
            let name = s.string_named(m)?.unwrap_or_default();
            rec.materials.push((name, s.i32_at(m + 8)?, s.i32_at(m + 12)?));
        }
    }
    let n = count(s, c + 0x20)?;
    rec.brush_sides = span(s, c + 0x28, n, 16, 7)?;
    let n = count(s, c + 0x30)?;
    rec.leaf_brush_nodes = span(s, c + 0x38, n, LEAF_BRUSH_NODE, 7)?;
    if let Some((t, n)) = rec.leaf_brush_nodes {
        for i in 0..n {
            let e = t + LEAF_BRUSH_NODE * i;
            let k = s.u16_at(e + 2)? as i16;
            if k > 0 {
                let at = s.records(e + 8, k as usize, 2, 1)?;
                rec.raw_leaf_lists.push((clip_ptr(s, e + 8, at)?, k as usize));
            } else {
                rec.raw_leaf_lists.push((ClipPtr::Null, 0));
            }
        }
    }
    let n = count(s, c + 0x40)?;
    s.records(c + 0x48, n, 2, 1)?;
    let n = count(s, c + 0x50)?;
    rec.brush_verts = span(s, c + 0x58, n, 12, 3)?;
    let n = count(s, c + 0x60)?;
    s.records(c + 0x68, n, 4, 3)?;
    let n = s.u16_at(c + 0x70)? as usize;
    rec.brushes = span(s, c + 0x78, n, CLIP_BRUSH, 15)?;
    if let Some((t, n)) = rec.brushes {
        for i in 0..n {
            let b = t + CLIP_BRUSH * i;
            let sides = s.records(b + 32, count(s, b + 28)?, 16, 7)?;
            rec.raw_side_lists.push(clip_ptr(s, b + 32, sides)?);
            let verts = s.records(b + 96, count(s, b + 88)?, 12, 3)?;
            rec.raw_vert_lists.push(clip_ptr(s, b + 96, verts)?);
        }
    }
    for f in [0x80, 0x88] {
        if s.ptr_at(c + f)?.is_inline() {
            return Err(crate::cursor::ReadError::Invalid("clip info bounds/contents (not read yet)", c + f));
        }
    }
    Ok(())
}

/// Resolves the clip map's references to its own arrays. The zone's VIRTUAL
/// offsets inside the clip map run ahead of the walk's by a constant on
/// retail maps (the glass work memory's block, about 0x180 on
/// `mp_nuketown_x`): measured from the lowest brush side reference (the
/// first side) and checked against the lowest vertex reference.
fn resolve_clip(s: &Stream, rec: &mut ClipMapRec) {
    let lowest = |v: &[ClipPtr]| v.iter().filter_map(|p| if let ClipPtr::Ref(r) = p { Some(*r) } else { None }).min();
    let virt = |span: Span| span.and_then(|(at, _)| s.vmap.virt_of(at));
    let side_delta = lowest(&rec.raw_side_lists).zip(virt(rec.brush_sides)).map(|(r, v)| r as i64 - v as i64);
    let vert_delta = lowest(&rec.raw_vert_lists).zip(virt(rec.brush_verts)).map(|(r, v)| r as i64 - v as i64);
    rec.virt_delta = match (side_delta, vert_delta) {
        (Some(a), Some(b)) if a == b => a,
        (Some(a), None) | (None, Some(a)) => a,
        _ => 0,
    };
    let d = rec.virt_delta;
    let file = |p: ClipPtr| match p {
        ClipPtr::Null => None,
        ClipPtr::Here(at) => Some(at),
        ClipPtr::Ref(r) => s.vmap.file_pos((r as i64 - d) as u64),
    };
    rec.brush_side_lists = rec.raw_side_lists.iter().map(|&p| file(p)).collect();
    rec.brush_vert_lists = rec.raw_vert_lists.iter().map(|&p| file(p)).collect();
    rec.leaf_brush_lists = rec.raw_leaf_lists.iter().map(|&(p, n)| file(p).map(|at| (at, n))).collect();
}

/// `clipMap_t` (0x2d0, x64-zt layout). Order read on `mp_nuketown_x`: the
/// name (a reference), the `ClipInfo`, static models (112: the model a
/// reference), nodes (16: a plane reference, two children), leafs (56),
/// vertices (12), triangles (3 u32s), the walkable-edge bits (one per
/// triangle edge, in u32s), partitions (16), AABB trees (32), sub-models
/// (96), the cluster visibility, the box brush, the dynamic entities'
/// definitions (152; the other lists are runtime) and the physics
/// constraints (208; the ropes are runtime).
pub fn clip_map(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let name = s.string(h)?;
    let mut rec = ClipMapRec { name: name.clone().unwrap_or_default(), header_at: h, ..Default::default() };
    clip_info(s, h + 0x10, &mut rec)?;
    if s.ptr_at(h + 0xa0)?.is_inline() {
        return Err(crate::cursor::ReadError::Invalid("clip map pInfo (not read yet)", h + 0xa0));
    }
    let n = count(s, h + 0xa8)?;
    if let Some(t) = s.records(h + 0xb0, n, CLIP_STATIC_MODEL, 7)? {
        rec.static_models_at = Some(t);
        for i in 0..n {
            let e = t + CLIP_STATIC_MODEL * i;
            let model = s.asset_named(AssetType::XModel, e + 8)?;
            if s.ptr_at(e + 96)?.is_inline() {
                return Err(crate::cursor::ReadError::Invalid("clip static model bones (not read yet)", e + 96));
            }
            rec.static_models.push(ClipStaticModel { model, at: e });
        }
    }
    let n = count(s, h + 0xb8)?;
    rec.nodes = span(s, h + 0xc0, n, 16, 7)?;
    let n = count(s, h + 0xc8)?;
    rec.leafs = span(s, h + 0xd0, n, CLIP_LEAF, 7)?;
    let n = count(s, h + 0xd8)?;
    rec.verts = span(s, h + 0xe0, n, 12, 3)?;
    let tris = count(s, h + 0xe8)?;
    rec.tri_indices = span(s, h + 0xf0, tris, 12, 3)?;
    s.array(h + 0xf8, (tris * 3).div_ceil(32) * 4, 3)?;
    let n = count(s, h + 0x100)?;
    rec.partitions = span(s, h + 0x108, n, 16, 3)?;
    let n = count(s, h + 0x110)?;
    rec.aabb_trees = span(s, h + 0x118, n, 32, 15)?;
    let n = count(s, h + 0x120)?;
    rec.cmodels = span(s, h + 0x128, n, CLIP_CMODEL, 7)?;
    if let Some((t, n)) = rec.cmodels {
        for i in 0..n {
            if s.ptr_at(t + CLIP_CMODEL * i + 32)?.is_inline() {
                return Err(crate::cursor::ReadError::Invalid("clip sub-model info (not read yet)", t + CLIP_CMODEL * i + 32));
            }
        }
    }
    let n = count(s, h + 0x130)? * count(s, h + 0x134)?;
    s.array(h + 0x138, n, 0)?;
    if s.ptr_at(h + 0x148)?.is_inline() {
        return Err(crate::cursor::ReadError::Invalid("clip map's map ents inline (not read yet)", h + 0x148));
    }
    if let Some(b) = s.array(h + 0x150, CLIP_BRUSH, 15)? {
        s.records(b + 32, count(s, b + 28)?, 16, 7)?;
        s.records(b + 96, count(s, b + 88)?, 12, 3)?;
    }
    // Dynamic entities: per list, `dynEntCount[list]` entries.
    for list in 0..2 {
        let n = s.u16_at(h + 0x1ba + 2 * list)? as usize;
        let defs = span(s, h + 0x1c8 + 8 * list, n, DYN_ENT_DEF, 7)?;
        if let Some((t, n)) = defs {
            for i in 0..n {
                let d = t + DYN_ENT_DEF * i;
                let model = s.asset_named(AssetType::XModel, d + 32)?;
                s.asset(AssetType::XModel, d + 40)?;
                for f in [56, 72, 88, 96, 104] {
                    s.asset(AssetType::Fx, d + f)?;
                }
                s.asset(AssetType::PhysPreset, d + 112)?;
                if list == 0 {
                    rec.dyn_ent_models.push(model);
                }
            }
        }
        rec.dyn_ent_defs[list] = defs;
    }
    // (The poses, client, server and collision lists are runtime.)
    let n = count(s, h + 0x228)?;
    s.records(h + 0x230, n, PHYS_CONSTRAINT, 7)?;
    resolve_clip(s, &mut rec);
    s.capture.world.clip_maps.push(rec);
    Ok(name)
}
