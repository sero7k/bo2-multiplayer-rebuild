//! bo2mp: a map's gameplay half in one small file (`<game>/<map>.iw4lmap`),
//! so a dedicated server can host a map from a game it does not have
//! installed: its collision (brushes, BSP, mesh tables, brush models, static
//! models' collision, trigger hulls), spawn points, entity string and the
//! placements the match reads from it. Nothing drawn or heard is kept.
//!
//! A pack is built from the operator's own install (`iw4l pack`), never
//! shipped. The file is `MAGIC`, the format version and the payload's size
//! (little-endian u32s), then the payload compressed with LZ4.

use asset_world::{
    ClipBrush, ClipBspLeaf, ClipBspNode, ClipCmodel, ClipCollision, ClipMapMaterial,
    ClipPlacedStaticModel, FlagDescriptor, IntermissionView, MapScriptStruct, MapTriggerHull,
    ScriptBrushModelPlacement, SpawnPoint,
};
use clipmap_iw4::{
    ClipAabbNode, ClipBorder, ClipMeshTables, ClipPartition, ClipStaticModel, XModelColl,
    XModelCollSurf, XModelCollTri,
};
use std::path::{Path, PathBuf};

pub const MAGIC: &[u8; 8] = b"IW4LMAP\0";
pub const VERSION: u32 = 1;
pub const EXTENSION: &str = "iw4lmap";

/// One map's gameplay data.
#[derive(Clone, Debug, Default)]
pub struct MapPack {
    /// Its game's prefix (`iw4`, `iw3`, `iw6`, ...).
    pub game: String,
    /// Its zone name (`mp_highrise`).
    pub map: String,
    /// Its entity string, as its game's reader gives it.
    pub entities: String,
    pub spawns: Vec<SpawnPoint>,
    pub intermission_view: Option<IntermissionView>,
    pub script_brush_models: Vec<ScriptBrushModelPlacement>,
    pub script_structs: Vec<MapScriptStruct>,
    pub flag_descriptors: Vec<FlagDescriptor>,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub world_bounds: Option<[f32; 6]>,
    pub collision: Option<ClipCollision>,
}

/// Where a map's pack is in a pack folder.
pub fn pack_path(dir: &Path, game: &str, map: &str) -> PathBuf {
    dir.join(game).join(format!("{map}.{EXTENSION}"))
}

/// Whether a path names a pack file.
pub fn is_pack_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION))
}

/// The pack folder a dedicated server reads (`IW4L_SERVER_PACKS`), when set.
pub fn packs_dir() -> Option<PathBuf> {
    std::env::var_os("IW4L_SERVER_PACKS")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// A map key (`iw4:mp_highrise`) as (game, map); a bare name is MW2's, as
/// the zone search takes it.
pub fn split_key(key: &str) -> (String, String) {
    match key.trim().to_ascii_lowercase().split_once(':') {
        Some((g, m)) => (g.to_owned(), m.to_owned()),
        None => ("iw4".to_owned(), key.trim().to_ascii_lowercase()),
    }
}

/// The pack for a map key in the server's pack folder, when there is one.
pub fn find_pack(key: &str) -> Option<PathBuf> {
    let dir = packs_dir()?;
    let (game, map) = split_key(key);
    let path = pack_path(&dir, &game, &map);
    path.is_file().then_some(path)
}

impl MapPack {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = W(Vec::new());
        w.str(&self.game);
        w.str(&self.map);
        w.str(&self.entities);
        w.len(self.spawns.len());
        for s in &self.spawns {
            w.str(&s.classname);
            w.v3(s.origin);
            w.v3(s.angles);
            w.str(&s.script_linkto);
            w.str(&s.script_destructable_area);
        }
        w.opt(self.intermission_view.as_ref(), |w, v| {
            w.v3(v.origin);
            w.v3(v.angles);
        });
        w.len(self.script_brush_models.len());
        for b in &self.script_brush_models {
            w.u32(b.source_ordinal);
            w.u32(b.cmodel_handle);
            w.v3(b.origin);
            w.v3(b.angles);
            w.str(&b.targetname);
            w.str(&b.gameobject);
            w.str(&b.script_exploder);
            w.opt(b.script_accumulate.as_ref(), |w, v| w.i32(*v));
            w.opt(b.script_threshold.as_ref(), |w, v| w.i32(*v));
            w.str(&b.script_destructable_area);
            w.str(&b.script_fxid);
        }
        w.len(self.script_structs.len());
        for s in &self.script_structs {
            w.str(&s.targetname);
            w.v3(s.origin);
            w.v3(s.angles);
            w.str(&s.target);
            w.opt(s.script_int.as_ref(), |w, v| w.i32(*v));
        }
        w.len(self.flag_descriptors.len());
        for f in &self.flag_descriptors {
            w.v3(f.origin);
            w.str(&f.script_linkname);
            w.str(&f.script_linkto);
        }
        w.v3(self.min);
        w.v3(self.max);
        w.opt(self.world_bounds.as_ref(), |w, b| {
            b.iter().for_each(|x| w.f32(*x))
        });
        w.opt(self.collision.as_ref(), encode_collision);
        w.0
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut r = R { b: bytes, at: 0 };
        let mut pack = MapPack {
            game: r.str()?,
            map: r.str()?,
            entities: r.str()?,
            ..Default::default()
        };
        for _ in 0..r.len()? {
            pack.spawns.push(SpawnPoint {
                classname: r.str()?,
                origin: r.v3()?,
                angles: r.v3()?,
                script_linkto: r.str()?,
                script_destructable_area: r.str()?,
            });
        }
        pack.intermission_view = r.opt(|r| {
            Ok(IntermissionView {
                origin: r.v3()?,
                angles: r.v3()?,
            })
        })?;
        for _ in 0..r.len()? {
            pack.script_brush_models.push(ScriptBrushModelPlacement {
                source_ordinal: r.u32()?,
                cmodel_handle: r.u32()?,
                origin: r.v3()?,
                angles: r.v3()?,
                targetname: r.str()?,
                gameobject: r.str()?,
                script_exploder: r.str()?,
                script_accumulate: r.opt(R::i32)?,
                script_threshold: r.opt(R::i32)?,
                script_destructable_area: r.str()?,
                script_fxid: r.str()?,
            });
        }
        for _ in 0..r.len()? {
            pack.script_structs.push(MapScriptStruct {
                targetname: r.str()?,
                origin: r.v3()?,
                angles: r.v3()?,
                target: r.str()?,
                script_int: r.opt(R::i32)?,
            });
        }
        for _ in 0..r.len()? {
            pack.flag_descriptors.push(FlagDescriptor {
                origin: r.v3()?,
                script_linkname: r.str()?,
                script_linkto: r.str()?,
            });
        }
        pack.min = r.v3()?;
        pack.max = r.v3()?;
        pack.world_bounds =
            r.opt(|r| Ok([r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?]))?;
        pack.collision = r.opt(decode_collision)?;
        if r.at != r.b.len() {
            return Err(format!(
                "{} bytes past the end of the pack",
                r.b.len() - r.at
            ));
        }
        Ok(pack)
    }

    /// The file's bytes: header, then the payload compressed.
    pub fn to_file_bytes(&self) -> Vec<u8> {
        let payload = self.encode();
        let packed = lz4_flex::compress(&payload);
        let mut out = Vec::with_capacity(16 + packed.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&packed);
        out
    }

    pub fn from_file_bytes(bytes: &[u8]) -> Result<Self, String> {
        let head = bytes.get(..16).ok_or("not a map pack (too short)")?;
        if &head[..8] != MAGIC {
            return Err("not a map pack".to_owned());
        }
        let version = u32::from_le_bytes(head[8..12].try_into().map_err(|_| "header")?);
        if version != VERSION {
            return Err(format!(
                "map pack version {version}; this build reads {VERSION} (build the packs again)"
            ));
        }
        let size = u32::from_le_bytes(head[12..16].try_into().map_err(|_| "header")?) as usize;
        let payload =
            lz4_flex::decompress(&bytes[16..], size).map_err(|e| format!("map pack: {e}"))?;
        Self::decode(&payload)
    }

    pub fn write(&self, path: &Path) -> Result<usize, String> {
        let bytes = self.to_file_bytes();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(bytes.len())
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_file_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }
}

fn encode_collision(w: &mut W, c: &ClipCollision) {
    w.len(c.brushes.len());
    for b in &c.brushes {
        w.len(b.planes.len());
        b.planes.iter().for_each(|p| w.v4(*p));
        w.u32(b.contents);
        w.u32s(&b.plane_surface_flags);
        w.u16(b.glass_encoded);
    }
    w.len(c.nodes.len());
    for n in &c.nodes {
        w.v4(n.plane);
        w.i32(n.children[0]);
        w.i32(n.children[1]);
    }
    w.len(c.leaves.len());
    for l in &c.leaves {
        w.u32(l.first_brush);
        w.u16(l.num_brushes);
        w.u16(l.first_coll_aabb_index);
        w.u16(l.coll_aabb_count);
    }
    w.u16s(&c.leafbrushes);
    let m = c.mesh.as_ref();
    w.len(m.verts.len());
    m.verts.iter().for_each(|v| w.v3(*v));
    w.u16s(&m.tri_indices);
    w.bytes(&m.tri_edge_is_walkable);
    w.u32s(&m.tri_surface_flags);
    w.u32s(&m.tri_content_flags);
    w.len(m.aabb_trees.len());
    for a in &m.aabb_trees {
        w.v3(a.origin);
        w.v3(a.half_size);
        w.u16(a.material_index);
        w.u16(a.child_count);
        w.i32(a.u);
    }
    w.len(m.partitions.len());
    for p in &m.partitions {
        w.u8(p.tri_count);
        w.i32(p.first_tri);
        w.u16(p.first_vert_segment);
        w.u8(p.border_count);
        w.u32(p.first_border);
    }
    w.len(m.borders.len());
    for b in &m.borders {
        w.v3(b.dist_eq);
        w.f32(b.z_base);
        w.f32(b.z_slope);
        w.f32(b.start);
        w.f32(b.length);
    }
    w.u16s(&m.aabb_roots);
    w.u16s(&c.tri_material_index);
    w.len(c.materials.len());
    for mat in &c.materials {
        w.str(&mat.name);
        w.u32(mat.surface_flags);
        w.u32(mat.content_flags);
    }
    w.len(c.cmodels.len());
    for cm in &c.cmodels {
        w.v3(cm.mins);
        w.v3(cm.maxs);
        w.f32(cm.radius);
        w.u32(cm.first_brush);
        w.u16(cm.num_brushes);
    }
    w.len(c.trigger_models.len());
    for hulls in &c.trigger_models {
        w.len(hulls.len());
        for h in hulls {
            w.v3(h.mid);
            w.v3(h.half);
            w.len(h.slabs.len());
            for (dir, mid, half) in &h.slabs {
                w.v3(*dir);
                w.f32(*mid);
                w.f32(*half);
            }
        }
    }
    w.len(c.static_models.len());
    for sm in &c.static_models {
        w.u32(sm.index);
        w.str(&sm.name);
        let s = &sm.model;
        w.v3(s.origin);
        s.inv_scaled_axis.iter().for_each(|a| w.v3(*a));
        w.v3(s.bounds_mid);
        w.v3(s.bounds_half);
        w.i16(s.coll.coll_lod);
        w.u32(s.coll.contents);
        w.len(s.coll.surfs.len());
        for surf in &s.coll.surfs {
            w.len(surf.tris.len());
            for t in &surf.tris {
                w.v4(t.plane);
                w.v4(t.svec);
                w.v4(t.tvec);
            }
            w.v3(surf.midpoint);
            w.v3(surf.half_size);
            w.i32(surf.bone_idx);
            w.u32(surf.contents);
            w.u32(surf.surf_flags);
        }
    }
}

fn decode_collision(r: &mut R) -> Result<ClipCollision, String> {
    let mut c = ClipCollision::default();
    for _ in 0..r.len()? {
        let n = r.len()?;
        let planes = (0..n).map(|_| r.v4()).collect::<Result<_, _>>()?;
        c.brushes.push(ClipBrush {
            planes,
            contents: r.u32()?,
            plane_surface_flags: r.u32s()?,
            glass_encoded: r.u16()?,
        });
    }
    for _ in 0..r.len()? {
        c.nodes.push(ClipBspNode {
            plane: r.v4()?,
            children: [r.i32()?, r.i32()?],
        });
    }
    for _ in 0..r.len()? {
        c.leaves.push(ClipBspLeaf {
            first_brush: r.u32()?,
            num_brushes: r.u16()?,
            first_coll_aabb_index: r.u16()?,
            coll_aabb_count: r.u16()?,
        });
    }
    c.leafbrushes = r.u16s()?;
    let mut m = ClipMeshTables::default();
    let n = r.len()?;
    m.verts = (0..n).map(|_| r.v3()).collect::<Result<_, _>>()?;
    m.tri_indices = r.u16s()?;
    m.tri_edge_is_walkable = r.bytes()?;
    m.tri_surface_flags = r.u32s()?;
    m.tri_content_flags = r.u32s()?;
    for _ in 0..r.len()? {
        m.aabb_trees.push(ClipAabbNode {
            origin: r.v3()?,
            half_size: r.v3()?,
            material_index: r.u16()?,
            child_count: r.u16()?,
            u: r.i32()?,
        });
    }
    for _ in 0..r.len()? {
        m.partitions.push(ClipPartition {
            tri_count: r.u8()?,
            first_tri: r.i32()?,
            first_vert_segment: r.u16()?,
            border_count: r.u8()?,
            first_border: r.u32()?,
        });
    }
    for _ in 0..r.len()? {
        m.borders.push(ClipBorder {
            dist_eq: r.v3()?,
            z_base: r.f32()?,
            z_slope: r.f32()?,
            start: r.f32()?,
            length: r.f32()?,
        });
    }
    m.aabb_roots = r.u16s()?;
    c.mesh = std::sync::Arc::new(m);
    c.tri_material_index = r.u16s()?;
    for _ in 0..r.len()? {
        c.materials.push(ClipMapMaterial {
            name: r.str()?,
            surface_flags: r.u32()?,
            content_flags: r.u32()?,
        });
    }
    for _ in 0..r.len()? {
        c.cmodels.push(ClipCmodel {
            mins: r.v3()?,
            maxs: r.v3()?,
            radius: r.f32()?,
            first_brush: r.u32()?,
            num_brushes: r.u16()?,
        });
    }
    for _ in 0..r.len()? {
        let mut hulls = Vec::new();
        for _ in 0..r.len()? {
            let (mid, half) = (r.v3()?, r.v3()?);
            let slabs = (0..r.len()?)
                .map(|_| Ok((r.v3()?, r.f32()?, r.f32()?)))
                .collect::<Result<_, String>>()?;
            hulls.push(MapTriggerHull { mid, half, slabs });
        }
        c.trigger_models.push(hulls);
    }
    for _ in 0..r.len()? {
        let index = r.u32()?;
        let name = r.str()?;
        let origin = r.v3()?;
        let inv_scaled_axis = [r.v3()?, r.v3()?, r.v3()?];
        let (bounds_mid, bounds_half) = (r.v3()?, r.v3()?);
        let (coll_lod, contents) = (r.i16()?, r.u32()?);
        let mut surfs = Vec::new();
        for _ in 0..r.len()? {
            let tris = (0..r.len()?)
                .map(|_| {
                    Ok(XModelCollTri {
                        plane: r.v4()?,
                        svec: r.v4()?,
                        tvec: r.v4()?,
                    })
                })
                .collect::<Result<_, String>>()?;
            surfs.push(XModelCollSurf {
                tris,
                midpoint: r.v3()?,
                half_size: r.v3()?,
                bone_idx: r.i32()?,
                contents: r.u32()?,
                surf_flags: r.u32()?,
            });
        }
        c.static_models.push(ClipPlacedStaticModel {
            index,
            name,
            model: ClipStaticModel {
                origin,
                inv_scaled_axis,
                bounds_mid,
                bounds_half,
                coll: XModelColl {
                    coll_lod,
                    contents,
                    surfs,
                },
            },
        });
    }
    Ok(c)
}

struct W(Vec<u8>);

impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i16(&mut self, v: i16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn len(&mut self, n: usize) {
        self.u32(n as u32);
    }
    fn v3(&mut self, v: [f32; 3]) {
        v.iter().for_each(|x| self.f32(*x));
    }
    fn v4(&mut self, v: [f32; 4]) {
        v.iter().for_each(|x| self.f32(*x));
    }
    fn bytes(&mut self, b: &[u8]) {
        self.len(b.len());
        self.0.extend_from_slice(b);
    }
    fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
    fn u16s(&mut self, v: &[u16]) {
        self.len(v.len());
        v.iter().for_each(|x| self.u16(*x));
    }
    fn u32s(&mut self, v: &[u32]) {
        self.len(v.len());
        v.iter().for_each(|x| self.u32(*x));
    }
    fn opt<T>(&mut self, v: Option<&T>, f: impl FnOnce(&mut Self, &T)) {
        match v {
            Some(v) => {
                self.u8(1);
                f(self, v);
            }
            None => self.u8(0),
        }
    }
}

struct R<'a> {
    b: &'a [u8],
    at: usize,
}

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self
            .at
            .checked_add(n)
            .filter(|e| *e <= self.b.len())
            .ok_or("map pack cut short")?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn arr<const N: usize>(&mut self) -> Result<[u8; N], String> {
        self.take(N)?
            .try_into()
            .map_err(|_| "map pack cut short".to_owned())
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.arr::<1>()?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.arr()?))
    }
    fn i16(&mut self) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.arr()?))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.arr()?))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.arr()?))
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.arr()?))
    }
    fn len(&mut self) -> Result<usize, String> {
        let n = self.u32()? as usize;
        // (No count can be larger than the bytes left: a damaged file fails
        // here instead of asking for gigabytes.)
        if n > self.b.len() - self.at {
            return Err("map pack: a count past the end of the file".to_owned());
        }
        Ok(n)
    }
    fn v3(&mut self) -> Result<[f32; 3], String> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }
    fn v4(&mut self) -> Result<[f32; 4], String> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }
    fn bytes(&mut self) -> Result<Vec<u8>, String> {
        let n = self.len()?;
        Ok(self.take(n)?.to_vec())
    }
    fn str(&mut self) -> Result<String, String> {
        String::from_utf8(self.bytes()?).map_err(|_| "map pack: text is not UTF-8".to_owned())
    }
    fn u16s(&mut self) -> Result<Vec<u16>, String> {
        (0..self.len()?).map(|_| self.u16()).collect()
    }
    fn u32s(&mut self) -> Result<Vec<u32>, String> {
        (0..self.len()?).map(|_| self.u32()).collect()
    }
    fn opt<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<Option<T>, String> {
        match self.u8()? {
            0 => Ok(None),
            1 => f(self).map(Some),
            n => Err(format!("map pack: bad option tag {n}")),
        }
    }
}
