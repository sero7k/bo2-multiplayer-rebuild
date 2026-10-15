//! bo2mp: a Modern Warfare III map, converted, kept on disk.
//!
//! Reading Rebirth Island's zone (257 MiB through the game's Oodle), its
//! 35 streamed tiles and 3,500 placed models' meshes, then baking its
//! light against 20 million triangles takes most of a minute; what comes
//! out is the same every time for the same install. So the converted map
//! (the world's vertices with their baked light, the placed models, the
//! entities with their spawns, and what its pictures are: the packages'
//! keys, not their pixels) is written to the game's cache
//! (`iw4l-artifacts/cache/jup_map`), and the collision built from it
//! beside it (`jup_clip`). The next load of the map reads those instead.
//!
//! An entry is found by a key over everything it was made from: the map's
//! zone files (the map's own and its tiles: name, size, time written), the
//! install's packages (the meshes' and pictures' bytes), this format's
//! number and the converting code's own text (so a change to the reader or
//! the converter makes a new entry, never a stale one). A file that does
//! not read back whole (cut short, another format) is a miss.

use std::path::Path;

use fastfile_jup::map::materials::{MapImage, MapMaterial as JupMaterial};
use fastfile_jup::map::terrain::VirtualTexture;
use fastfile_t7::map::{MapModel, ModelSurface, StaticModel, T7Map, WorldSurface, WorldVertex};

/// The layout's number (raised when what is written changes).
const FORMAT: u32 = 1;
const MAGIC: &[u8; 8] = b"BO2JUPM\0";
/// The cache kinds (folders under `iw4l-artifacts/cache`).
pub(super) const MAP_KIND: &str = "jup_map";
pub(super) const CLIP_KIND: &str = "jup_clip";

/// The text of the code a cached map comes out of (its fingerprint is in
/// every key).
const MAP_CODE: &[&str] = &[
    include_str!("jup_cache.rs"),
    include_str!("jup_map.rs"),
    include_str!("jup_light.rs"),
    include_str!("jup_terrain.rs"),
    include_str!("../../../fastfile_jup/src/map/mod.rs"),
    include_str!("../../../fastfile_jup/src/map/index.rs"),
    include_str!("../../../fastfile_jup/src/map/world.rs"),
    include_str!("../../../fastfile_jup/src/map/tiles.rs"),
    include_str!("../../../fastfile_jup/src/map/models.rs"),
    include_str!("../../../fastfile_jup/src/map/materials.rs"),
    include_str!("../../../fastfile_jup/src/map/terrain.rs"),
    include_str!("../../../fastfile_jup/src/map/ents.rs"),
    include_str!("../../../fastfile_jup/src/surfs.rs"),
    include_str!("../../../fastfile_jup/src/image.rs"),
    include_str!("../../../fastfile_jup/src/container.rs"),
];

/// The text of the code the collision comes out of (with the map's).
const CLIP_CODE: &[&str] = &[
    include_str!("t7_map.rs"),
    include_str!("../../../asset_world/src/world_iw7.rs"),
    include_str!("../../../asset_world/src/clip_collision.rs"),
    include_str!("../../../clipmap_iw4/src/mesh.rs"),
];

fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

const FNV_START: u64 = 0xcbf2_9ce4_8422_2325;

/// The cache keys of a map zone: (map's, collision's). `None` when the
/// zone's folder cannot be listed.
pub(super) fn keys(zone: &Path) -> Option<(String, String)> {
    let dir = zone.parent()?;
    let map = zone.file_stem()?.to_string_lossy().to_ascii_lowercase();
    let mut files: Vec<(String, u64, u64)> = Vec::new();
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let name = e.file_name().to_string_lossy().to_ascii_lowercase();
        let ours = name.starts_with(&map) && name.ends_with(".ff");
        if !(ours || name.ends_with(".xpak")) {
            continue;
        }
        let Ok(meta) = e.metadata() else { continue };
        let written = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs());
        files.push((name, meta.len(), written));
    }
    files.sort();
    let mut h = fnv(FNV_START, &FORMAT.to_le_bytes());
    h = fnv(h, map.as_bytes());
    for (name, len, written) in &files {
        h = fnv(h, name.as_bytes());
        h = fnv(h, &len.to_le_bytes());
        h = fnv(h, &written.to_le_bytes());
    }
    for code in MAP_CODE {
        h = fnv(h, code.as_bytes());
    }
    let mut c = h;
    for code in CLIP_CODE {
        c = fnv(c, code.as_bytes());
    }
    let clean: String = map.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
    Some((format!("{clean}-{h:016x}"), format!("{clean}-{c:016x}")))
}

/// A converted map: the Black Ops III form the builder reads (no pictures'
/// pixels, no materials yet) and what its pictures are.
pub(super) struct Converted {
    pub t7: T7Map,
    pub materials: Vec<JupMaterial>,
    pub images: Vec<MapImage>,
    pub sky_images: Vec<MapImage>,
    pub virtual_texture: Option<VirtualTexture>,
    pub has_terrain: bool,
    /// The report lines of the conversion.
    pub report: Vec<String>,
}

// ---------------------------------------------------------------- writing

#[derive(Default)]
pub(super) struct W(Vec<u8>);

impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn len(&mut self, n: usize) {
        self.u64(n as u64);
    }
    fn f2(&mut self, v: [f32; 2]) {
        v.iter().for_each(|&x| self.f32(x));
    }
    fn f3(&mut self, v: [f32; 3]) {
        v.iter().for_each(|&x| self.f32(x));
    }
    fn bool(&mut self, v: bool) {
        self.u8(u8::from(v));
    }
    fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        self.len(b.len());
        self.0.extend_from_slice(b);
    }
    fn opt_index(&mut self, v: Option<usize>) {
        self.u64(v.map_or(u64::MAX, |v| v as u64));
    }
    fn f3s(&mut self, v: &[[f32; 3]]) {
        self.len(v.len());
        self.0.reserve(v.len() * 12);
        v.iter().for_each(|&p| self.f3(p));
    }
    fn f2s(&mut self, v: &[[f32; 2]]) {
        self.len(v.len());
        v.iter().for_each(|&p| self.f2(p));
    }
    fn tris16(&mut self, v: &[[u16; 3]]) {
        self.len(v.len());
        v.iter().flatten().for_each(|&i| self.u16(i));
    }
    fn bools(&mut self, v: &[bool]) {
        self.len(v.len());
        v.iter().for_each(|&b| self.bool(b));
    }
}

// ---------------------------------------------------------------- reading

pub(super) struct R<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        let s = self.b.get(self.at..end)?;
        self.at = end;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    /// A count of items of at least `each` bytes (never more than the bytes
    /// left could hold).
    fn len(&mut self, each: usize) -> Option<usize> {
        let n = usize::try_from(self.u64()?).ok()?;
        (n.checked_mul(each.max(1))? <= self.b.len() - self.at).then_some(n)
    }
    fn f2(&mut self) -> Option<[f32; 2]> {
        Some([self.f32()?, self.f32()?])
    }
    fn f3(&mut self) -> Option<[f32; 3]> {
        Some([self.f32()?, self.f32()?, self.f32()?])
    }
    fn bool(&mut self) -> Option<bool> {
        Some(self.u8()? != 0)
    }
    fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = self.len(1)?;
        Some(self.take(n)?.to_vec())
    }
    fn str(&mut self) -> Option<String> {
        String::from_utf8(self.bytes()?).ok()
    }
    fn opt_index(&mut self) -> Option<Option<usize>> {
        let v = self.u64()?;
        Some((v != u64::MAX).then_some(v as usize))
    }
    fn vec<T>(&mut self, each: usize, mut item: impl FnMut(&mut Self) -> Option<T>) -> Option<Vec<T>> {
        let n = self.len(each)?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(item(self)?);
        }
        Some(out)
    }
    fn f3s(&mut self) -> Option<Vec<[f32; 3]>> {
        self.vec(12, Self::f3)
    }
    fn f2s(&mut self) -> Option<Vec<[f32; 2]>> {
        self.vec(8, Self::f2)
    }
    fn tris16(&mut self) -> Option<Vec<[u16; 3]>> {
        self.vec(6, |r| Some([r.u16()?, r.u16()?, r.u16()?]))
    }
    fn bools(&mut self) -> Option<Vec<bool>> {
        self.vec(1, Self::bool)
    }
}

// ------------------------------------------------------------ the records

fn put_model_surface(w: &mut W, s: &ModelSurface) {
    w.f3s(&s.positions);
    w.f3s(&s.normals);
    w.f2s(&s.uvs);
    w.tris16(&s.tris);
    w.opt_index(s.material);
}

fn get_model_surface(r: &mut R) -> Option<ModelSurface> {
    Some(ModelSurface {
        positions: r.f3s()?,
        normals: r.f3s()?,
        uvs: r.f2s()?,
        tris: r.tris16()?,
        material: r.opt_index()?,
    })
}

fn put_image(w: &mut W, i: &MapImage) {
    w.u64(i.hash);
    w.u16(i.width);
    w.u16(i.height);
    w.u8(i.format);
    w.len(i.groups.len());
    for g in &i.groups {
        w.u64(g.key);
        w.u32(g.end);
    }
    match &i.inline {
        Some(b) => {
            w.bool(true);
            w.bytes(b);
        }
        None => w.bool(false),
    }
}

fn get_image(r: &mut R) -> Option<MapImage> {
    Some(MapImage {
        hash: r.u64()?,
        width: r.u16()?,
        height: r.u16()?,
        format: r.u8()?,
        groups: r.vec(12, |r| Some(fastfile_jup::image::MipGroup { key: r.u64()?, end: r.u32()? }))?,
        inline: if r.bool()? { Some(r.bytes()?) } else { None },
    })
}

/// The converted map as bytes (before compression).
pub(super) fn encode(c: &Converted) -> Vec<u8> {
    let mut w = W::default();
    let t = &c.t7;
    w.len(c.report.len());
    c.report.iter().for_each(|l| w.str(l));
    w.str(&t.name);
    w.len(t.vertices.len());
    for v in &t.vertices {
        w.f3(v.xyz);
        w.u32(v.color);
        w.f2(v.uv);
        w.f3(v.normal);
        w.f3(v.tangent);
    }
    w.len(t.indices.len());
    t.indices.iter().for_each(|&i| w.u16(i));
    w.len(t.surfaces.len());
    for s in &t.surfaces {
        w.u32(s.first_vertex);
        w.u32(s.vertex_count);
        w.u16(s.tri_count);
        w.u32(s.base_index);
        w.opt_index(s.material);
        w.u8(s.flags);
        w.f3(s.mins);
        w.f3(s.maxs);
    }
    w.len(t.models.len());
    for m in &t.models {
        w.str(&m.name);
        w.len(m.surfaces.len());
        m.surfaces.iter().for_each(|s| put_model_surface(&mut w, s));
        w.len(m.coarse.len());
        m.coarse.iter().for_each(|s| put_model_surface(&mut w, s));
    }
    w.len(t.static_models.len());
    for s in &t.static_models {
        w.u64(s.model as u64);
        w.f3(s.origin);
        s.axis.iter().for_each(|&a| w.f3(a));
        w.f32(s.scale);
        w.f32(s.cull_dist);
    }
    w.str(&t.entities);
    w.len(t.trigger_models.len());
    for hulls in &t.trigger_models {
        w.len(hulls.len());
        for (mid, half) in hulls {
            w.f3(*mid);
            w.f3(*half);
        }
    }
    w.f3(t.mins);
    w.f3(t.maxs);
    w.bools(&t.no_collision);
    w.bools(&t.static_no_collision);
    match t.baked_ambient {
        Some(a) => {
            w.bool(true);
            w.f3(a);
        }
        None => w.bool(false),
    }
    w.len(t.static_light.len());
    t.static_light.iter().for_each(|l| w.0.extend_from_slice(l));
    // What the pictures are.
    w.len(c.materials.len());
    for m in &c.materials {
        w.u64(m.hash);
        w.len(m.images.len());
        m.images.iter().for_each(|&i| w.u64(i as u64));
        w.opt_index(m.colour);
    }
    w.len(c.images.len());
    c.images.iter().for_each(|i| put_image(&mut w, i));
    w.len(c.sky_images.len());
    c.sky_images.iter().for_each(|i| put_image(&mut w, i));
    match &c.virtual_texture {
        Some(vt) => {
            w.bool(true);
            w.u32(vt.levels);
            w.len(vt.tiles.len());
            for t in &vt.tiles {
                match t {
                    Some(i) => {
                        w.bool(true);
                        put_image(&mut w, i);
                    }
                    None => w.bool(false),
                }
            }
            w.f2(vt.origin);
            w.f32(vt.extent);
        }
        None => w.bool(false),
    }
    w.bool(c.has_terrain);
    w.0
}

/// The converted map back from its bytes; `None` when they do not read
/// whole.
pub(super) fn decode(b: &[u8]) -> Option<Converted> {
    let mut r = R { b, at: 0 };
    let report = r.vec(8, R::str)?;
    let mut t = T7Map::default();
    t.name = r.str()?;
    t.vertices = r.vec(52, |r| {
        Some(WorldVertex { xyz: r.f3()?, color: r.u32()?, uv: r.f2()?, normal: r.f3()?, tangent: r.f3()? })
    })?;
    t.indices = r.vec(2, R::u16)?;
    t.surfaces = r.vec(49, |r| {
        Some(WorldSurface {
            first_vertex: r.u32()?,
            vertex_count: r.u32()?,
            tri_count: r.u16()?,
            base_index: r.u32()?,
            material: r.opt_index()?,
            flags: r.u8()?,
            mins: r.f3()?,
            maxs: r.f3()?,
        })
    })?;
    t.models = r.vec(24, |r| {
        Some(MapModel {
            name: r.str()?,
            surfaces: r.vec(40, get_model_surface)?,
            coarse: r.vec(40, get_model_surface)?,
        })
    })?;
    t.static_models = r.vec(64, |r| {
        Some(StaticModel {
            model: usize::try_from(r.u64()?).ok()?,
            origin: r.f3()?,
            axis: [r.f3()?, r.f3()?, r.f3()?],
            scale: r.f32()?,
            cull_dist: r.f32()?,
        })
    })?;
    if t.static_models.iter().any(|s| s.model >= t.models.len()) {
        return None;
    }
    t.entities = r.str()?;
    t.trigger_models = r.vec(8, |r| r.vec(24, |r| Some((r.f3()?, r.f3()?))))?;
    t.mins = r.f3()?;
    t.maxs = r.f3()?;
    t.no_collision = r.bools()?;
    t.static_no_collision = r.bools()?;
    t.baked_ambient = if r.bool()? { Some(r.f3()?) } else { None };
    t.static_light = r.vec(4, |r| Some([r.u8()?, r.u8()?, r.u8()?, r.u8()?]))?;
    let materials = r.vec(17, |r| {
        Some(JupMaterial {
            hash: r.u64()?,
            images: r.vec(8, |r| usize::try_from(r.u64()?).ok())?,
            colour: r.opt_index()?,
        })
    })?;
    let images = r.vec(22, get_image)?;
    let sky_images = r.vec(22, get_image)?;
    let virtual_texture = if r.bool()? {
        Some(VirtualTexture {
            levels: r.u32()?,
            tiles: r.vec(1, |r| Some(if r.bool()? { Some(get_image(r)?) } else { None }))?,
            origin: r.f2()?,
            extent: r.f32()?,
        })
    } else {
        None
    };
    let has_terrain = r.bool()?;
    // (Indices into what was read stay in range.)
    let vertices = t.vertices.len() as u64;
    let indices = t.indices.len() as u64;
    let bad_surface = t.surfaces.iter().any(|s| {
        u64::from(s.first_vertex) + u64::from(s.vertex_count) > vertices
            || u64::from(s.base_index) + 3 * u64::from(s.tri_count) > indices
            || s.material.is_some_and(|m| m > materials.len())
    });
    let bad_material = materials.iter().any(|m| m.images.iter().chain(m.colour.iter()).any(|&i| i >= images.len()));
    if bad_surface || bad_material || r.at != b.len() {
        return None;
    }
    Some(Converted { t7: t, materials, images, sky_images, virtual_texture, has_terrain, report })
}

// ----------------------------------------------------------- the files

/// Bytes compressed in pieces (several at once), behind the magic, the
/// format and the key.
pub(super) fn pack(key: &str, raw: &[u8]) -> Vec<u8> {
    const PIECE: usize = 8 << 20;
    let pieces: Vec<&[u8]> = raw.chunks(PIECE).collect();
    let packed: Vec<Vec<u8>> = std::thread::scope(|s| {
        let jobs: Vec<_> = pieces.iter().map(|p| s.spawn(move || lz4_flex::block::compress(p))).collect();
        jobs.into_iter().map(|j| j.join().unwrap_or_default()).collect()
    });
    let mut out = Vec::with_capacity(packed.iter().map(Vec::len).sum::<usize>() + 64 + 16 * pieces.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT.to_le_bytes());
    out.extend_from_slice(&fnv(FNV_START, key.as_bytes()).to_le_bytes());
    out.extend_from_slice(&(pieces.len() as u64).to_le_bytes());
    for (p, c) in pieces.iter().zip(&packed) {
        out.extend_from_slice(&(p.len() as u64).to_le_bytes());
        out.extend_from_slice(&(c.len() as u64).to_le_bytes());
    }
    for c in &packed {
        out.extend_from_slice(c);
    }
    out
}

/// What [`pack`] made, unpacked (several pieces at once); `None` when it
/// is not whole or is another key's.
pub(super) fn unpack(key: &str, b: &[u8]) -> Option<Vec<u8>> {
    let mut r = R { b, at: 0 };
    if r.take(8)? != MAGIC || r.u32()? != FORMAT || r.u64()? != fnv(FNV_START, key.as_bytes()) {
        return None;
    }
    let n = r.len(16)?;
    let sizes: Vec<(usize, usize)> =
        (0..n).map(|_| Some((usize::try_from(r.u64()?).ok()?, usize::try_from(r.u64()?).ok()?))).collect::<Option<_>>()?;
    let mut pieces = Vec::with_capacity(n);
    for &(raw, packed) in &sizes {
        pieces.push((raw, r.take(packed)?));
    }
    if r.at != b.len() {
        return None;
    }
    let total: usize = sizes.iter().map(|s| s.0).sum();
    let mut out = vec![0u8; total];
    let ok = std::thread::scope(|s| {
        let mut rest = out.as_mut_slice();
        let mut jobs = Vec::new();
        for &(raw, packed) in &pieces {
            let (here, after) = rest.split_at_mut(raw);
            rest = after;
            jobs.push(s.spawn(move || lz4_flex::block::decompress_into(packed, here).is_ok_and(|n| n == raw)));
        }
        jobs.into_iter().all(|j| j.join().unwrap_or(false))
    });
    ok.then_some(out)
}

/// Reads a cache entry and unpacks it.
pub(super) fn read(kind: &str, key: &str) -> Option<Vec<u8>> {
    let bytes = asset_transport::cache_get(kind, key)?;
    unpack(key, &bytes)
}

/// Writes a cache entry (the map's other entries of this kind removed),
/// on a thread of its own: `encode` runs there.
pub(super) fn write_later(kind: &'static str, key: String, encode: impl FnOnce() -> Vec<u8> + Send + 'static) {
    let spawned = std::thread::Builder::new().name(format!("{kind} cache write")).spawn(move || {
        let started = std::time::Instant::now();
        let raw = encode();
        let packed = pack(&key, &raw);
        let map = key.rsplit_once('-').map_or(key.as_str(), |(m, _)| m);
        asset_transport::cache_prune(kind, &format!("{map}-"), &key);
        match asset_transport::cache_put(kind, &key, &packed) {
            Ok(()) => diag::info!(
                World,
                "bo2mp jup cache: {kind}/{key} written, {:.1} MiB ({:.1} MiB unpacked) in {:.1}s",
                packed.len() as f64 / (1024.0 * 1024.0),
                raw.len() as f64 / (1024.0 * 1024.0),
                started.elapsed().as_secs_f32()
            ),
            Err(e) => diag::warn!(World, "bo2mp jup cache: {kind}/{key} not written: {e}"),
        }
    });
    if let Err(e) = spawned {
        diag::warn!(World, "bo2mp jup cache: {kind} not written (no thread: {e})");
    }
}

// ------------------------------------------------------- the collision

/// The collision as bytes (before compression).
pub(super) fn encode_clip(c: &asset_world::ClipCollision) -> Vec<u8> {
    let mut w = W::default();
    w.len(c.brushes.len());
    for b in &c.brushes {
        w.len(b.planes.len());
        b.planes.iter().flatten().for_each(|&x| w.f32(x));
        w.u32(b.contents);
        w.len(b.plane_surface_flags.len());
        b.plane_surface_flags.iter().for_each(|&x| w.u32(x));
        w.u16(b.glass_encoded);
    }
    w.len(c.nodes.len());
    for n in &c.nodes {
        n.plane.iter().for_each(|&x| w.f32(x));
        n.children.iter().for_each(|&x| w.u32(x as u32));
    }
    w.len(c.leaves.len());
    for l in &c.leaves {
        w.u32(l.first_brush);
        w.u16(l.num_brushes);
        w.u16(l.first_coll_aabb_index);
        w.u16(l.coll_aabb_count);
    }
    w.len(c.leafbrushes.len());
    c.leafbrushes.iter().for_each(|&x| w.u16(x));
    let m = &c.mesh;
    w.f3s(&m.verts);
    w.len(m.tri_indices.len());
    m.tri_indices.iter().for_each(|&x| w.u16(x));
    w.bytes(&m.tri_edge_is_walkable);
    w.len(m.tri_surface_flags.len());
    m.tri_surface_flags.iter().for_each(|&x| w.u32(x));
    w.len(m.tri_content_flags.len());
    m.tri_content_flags.iter().for_each(|&x| w.u32(x));
    w.len(m.aabb_trees.len());
    for a in &m.aabb_trees {
        w.f3(a.origin);
        w.f3(a.half_size);
        w.u16(a.material_index);
        w.u16(a.child_count);
        w.u32(a.u as u32);
    }
    w.len(m.partitions.len());
    for p in &m.partitions {
        w.u8(p.tri_count);
        w.u32(p.first_tri as u32);
        w.u16(p.first_vert_segment);
        w.u8(p.border_count);
        w.u32(p.first_border);
    }
    w.len(m.borders.len());
    for b in &m.borders {
        w.f3(b.dist_eq);
        w.f32(b.z_base);
        w.f32(b.z_slope);
        w.f32(b.start);
        w.f32(b.length);
    }
    w.len(m.aabb_roots.len());
    m.aabb_roots.iter().for_each(|&x| w.u16(x));
    w.len(c.tri_material_index.len());
    c.tri_material_index.iter().for_each(|&x| w.u16(x));
    w.len(c.materials.len());
    for m in &c.materials {
        w.str(&m.name);
        w.u32(m.surface_flags);
        w.u32(m.content_flags);
    }
    w.len(c.cmodels.len());
    for m in &c.cmodels {
        w.f3(m.mins);
        w.f3(m.maxs);
        w.f32(m.radius);
        w.u32(m.first_brush);
        w.u16(m.num_brushes);
    }
    w.0
}

/// The collision back from its bytes (the trigger models and placed
/// models' records are the map's, set by the builder).
pub(super) fn decode_clip(b: &[u8]) -> Option<asset_world::ClipCollision> {
    use clipmap_iw4::{ClipAabbNode, ClipBorder, ClipMeshTables, ClipPartition};
    let mut r = R { b, at: 0 };
    let mut c = asset_world::ClipCollision::default();
    c.brushes = r.vec(24, |r| {
        Some(asset_world::ClipBrush {
            planes: r.vec(16, |r| Some([r.f32()?, r.f32()?, r.f32()?, r.f32()?]))?,
            contents: r.u32()?,
            plane_surface_flags: r.vec(4, R::u32)?,
            glass_encoded: r.u16()?,
        })
    })?;
    c.nodes = r.vec(24, |r| {
        Some(asset_world::ClipBspNode {
            plane: [r.f32()?, r.f32()?, r.f32()?, r.f32()?],
            children: [r.u32()? as i32, r.u32()? as i32],
        })
    })?;
    c.leaves = r.vec(10, |r| {
        Some(asset_world::ClipBspLeaf {
            first_brush: r.u32()?,
            num_brushes: r.u16()?,
            first_coll_aabb_index: r.u16()?,
            coll_aabb_count: r.u16()?,
        })
    })?;
    c.leafbrushes = r.vec(2, R::u16)?;
    let mut m = ClipMeshTables {
        verts: r.f3s()?,
        tri_indices: r.vec(2, R::u16)?,
        tri_edge_is_walkable: r.bytes()?,
        tri_surface_flags: r.vec(4, R::u32)?,
        tri_content_flags: r.vec(4, R::u32)?,
        ..Default::default()
    };
    m.aabb_trees = r.vec(32, |r| {
        Some(ClipAabbNode {
            origin: r.f3()?,
            half_size: r.f3()?,
            material_index: r.u16()?,
            child_count: r.u16()?,
            u: r.u32()? as i32,
        })
    })?;
    m.partitions = r.vec(12, |r| {
        Some(ClipPartition {
            tri_count: r.u8()?,
            first_tri: r.u32()? as i32,
            first_vert_segment: r.u16()?,
            border_count: r.u8()?,
            first_border: r.u32()?,
        })
    })?;
    m.borders = r.vec(24, |r| {
        Some(ClipBorder { dist_eq: r.f3()?, z_base: r.f32()?, z_slope: r.f32()?, start: r.f32()?, length: r.f32()? })
    })?;
    m.aabb_roots = r.vec(2, R::u16)?;
    c.mesh = std::sync::Arc::new(m);
    c.tri_material_index = r.vec(2, R::u16)?;
    c.materials = r.vec(16, |r| {
        Some(asset_world::ClipMapMaterial { name: r.str()?, surface_flags: r.u32()?, content_flags: r.u32()? })
    })?;
    c.cmodels = r.vec(34, |r| {
        Some(asset_world::ClipCmodel {
            mins: r.f3()?,
            maxs: r.f3()?,
            radius: r.f32()?,
            first_brush: r.u32()?,
            num_brushes: r.u16()?,
        })
    })?;
    (r.at == b.len()).then_some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_reads_back_as_written() {
        let mut t7 = T7Map::default();
        t7.name = "mp_test".into();
        t7.vertices = vec![
            WorldVertex { xyz: [1.0, 2.0, 3.0], color: 0xFF80_8080, uv: [0.5, 0.25], normal: [0.0, 0.0, 1.0], tangent: [1.0, 0.0, 0.0] };
            3
        ];
        t7.indices = vec![0, 1, 2];
        t7.surfaces = vec![WorldSurface { first_vertex: 0, vertex_count: 3, tri_count: 1, base_index: 0, material: Some(0), flags: 0, mins: [0.0; 3], maxs: [1.0; 3] }];
        let surface = ModelSurface { positions: vec![[0.0; 3]; 3], normals: vec![[0.0, 0.0, 1.0]; 3], uvs: vec![[0.0; 2]; 3], tris: vec![[0, 1, 2]], material: None };
        t7.models = vec![MapModel { name: "m".into(), surfaces: vec![surface.clone()], coarse: vec![surface] }];
        t7.static_models = vec![StaticModel { model: 0, origin: [4.0; 3], axis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], scale: 2.0, cull_dist: 1500.0 }];
        t7.entities = "{\n\"classname\" \"worldspawn\"\n}\n".into();
        t7.no_collision = vec![false];
        t7.static_no_collision = vec![true];
        t7.baked_ambient = Some([0.4, 0.43, 0.5]);
        t7.static_light = vec![[1, 2, 3, 0]];
        let image = MapImage { hash: 7, width: 4, height: 4, format: 0x25, groups: vec![fastfile_jup::image::MipGroup { key: 9, end: 8 }], inline: Some(vec![1, 2]) };
        let c = Converted {
            t7,
            materials: vec![JupMaterial { hash: 5, images: vec![0], colour: Some(0) }],
            images: vec![image.clone()],
            sky_images: vec![image.clone()],
            virtual_texture: Some(VirtualTexture { levels: 2, tiles: vec![None, Some(image)], origin: [1.0, 2.0], extent: 3.0 }),
            has_terrain: true,
            report: vec!["line".into()],
        };
        let raw = encode(&c);
        let packed = pack("mp_test-1", &raw);
        assert!(unpack("mp_test-2", &packed).is_none());
        let back = decode(&unpack("mp_test-1", &packed).unwrap()).unwrap();
        assert_eq!(encode(&back), raw);
        assert!(decode(&raw[..raw.len() - 1]).is_none());
    }

    #[test]
    fn a_collision_reads_back_as_written() {
        let tris: Vec<asset_world::world_iw7::SoupTri> = (0..100)
            .map(|i| {
                let x = i as f32 * 10.0;
                asset_world::world_iw7::SoupTri { p: [[x, 0.0, 0.0], [x + 8.0, 0.0, 0.0], [x, 8.0, 0.0]], surface_flags: 0, contents: 1 }
            })
            .collect();
        let clip = asset_world::world_iw7::build_soup_clip_collision(&tris, &[]);
        let raw = encode_clip(&clip);
        let back = decode_clip(&raw).unwrap();
        assert_eq!(encode_clip(&back), raw);
    }
}
