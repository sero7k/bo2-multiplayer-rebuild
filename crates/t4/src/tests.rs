//! Synthetic World at War zones: the parser and the map conversion end to
//! end. Struct offsets come from the generated schema by member name, so
//! these check the stream walk (inline data order, block positions and
//! references), not the schema itself.

use crate::convert::to_iw3_map;
use crate::zone::schema::schema;
use crate::zone::{ParseOptions, Zone};

const INLINE: u32 = 0xFFFF_FFFF;
const BLOCK_VIRTUAL: u32 = 4;
const ASSET_CLIPMAP_PVS: u32 = 12;
const ASSET_MAP_ENTS: u32 = 16;
const ASSET_GFX_WORLD: u32 = 17;

/// A reference to data at `offset` in the virtual block.
fn reference(offset: u32) -> u32 {
    ((BLOCK_VIRTUAL << 29) | offset) + 1
}

/// The offset of `path` (`a::b`) within struct `ty`.
fn offset(ty: &str, path: &str) -> usize {
    let s = schema();
    let mut ty = ty.to_owned();
    let mut off = 0;
    for seg in path.split("::") {
        let m = s.types[&ty].members.iter().find(|m| m.name.as_deref() == Some(seg)).unwrap_or_else(|| panic!("{ty}::{seg}"));
        off += m.offset as usize;
        ty = s.resolve(&m.ty).base;
    }
    off
}

/// A struct image of `ty`, zeroed.
fn image(ty: &str) -> Vec<u8> {
    vec![0; schema().types[ty].size as usize]
}

fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

fn put_f(b: &mut [u8], at: usize, v: f32) {
    put(b, at, v.to_bits());
}

/// The zone stream after the XFile header, tracking the virtual block.
struct Stream {
    bytes: Vec<u8>,
    virt: u32,
}

impl Stream {
    fn new(asset_types: &[u32]) -> Stream {
        let mut s = Stream { bytes: vec![0; 36], virt: 0 };
        // XAssetList: no script strings, the asset entries inline.
        for v in [0, 0, asset_types.len() as u32, INLINE] {
            s.bytes.extend_from_slice(&v.to_le_bytes());
        }
        let mut entries = Vec::new();
        for &t in asset_types {
            entries.extend_from_slice(&t.to_le_bytes());
            entries.extend_from_slice(&INLINE.to_le_bytes());
        }
        s.virt_data(4, &entries);
        s
    }

    /// Data placed in the temp block (an asset's header).
    fn temp(&mut self, data: &[u8]) {
        self.bytes.extend_from_slice(data);
    }

    /// Data placed in the virtual block; returns its offset there.
    fn virt_data(&mut self, align: u32, data: &[u8]) -> u32 {
        self.virt = (self.virt + align - 1) & !(align - 1);
        let at = self.virt;
        self.bytes.extend_from_slice(data);
        self.virt += data.len() as u32;
        at
    }

    fn string(&mut self, s: &str) -> u32 {
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        self.virt_data(1, &b)
    }
}

/// A clipMap with two planes, one brush side on the second and one brush,
/// then its entities; a GfxWorld with one triangle.
fn synthetic_zone() -> Vec<u8> {
    let mut z = Stream::new(&[ASSET_CLIPMAP_PVS, ASSET_MAP_ENTS, ASSET_GFX_WORLD]);

    // clipMap_t
    let mut c = image("clipMap_t");
    put(&mut c, offset("clipMap_t", "name"), INLINE);
    put(&mut c, offset("clipMap_t", "planeCount"), 2);
    put(&mut c, offset("clipMap_t", "planes"), INLINE);
    put(&mut c, offset("clipMap_t", "numBrushSides"), 1);
    put(&mut c, offset("clipMap_t", "brushsides"), INLINE);
    c[offset("clipMap_t", "numBrushes")..][..2].copy_from_slice(&1u16.to_le_bytes());
    put(&mut c, offset("clipMap_t", "brushes"), INLINE);
    z.temp(&c);
    z.string("maps/mp/mp_test.d3dbsp");
    let mut planes = Vec::new();
    for (n, d) in [([1.0f32, 0.0, 0.0], 8.0f32), ([0.6, 0.8, 0.0], 32.0)] {
        let mut p = image("cplane_s");
        for k in 0..3 {
            put_f(&mut p, k * 4, n[k]);
        }
        put_f(&mut p, 12, d);
        planes.extend(p);
    }
    let planes_at = z.virt_data(4, &planes);
    let mut side = image("cbrushside_t");
    put(&mut side, offset("cbrushside_t", "plane"), reference(planes_at + 20));
    put(&mut side, offset("cbrushside_t", "materialNum"), 0);
    let side_at = z.virt_data(4, &side);
    let mut brush = image("cbrush_t");
    for k in 0..3 {
        put_f(&mut brush, offset("cbrush_t", "mins") + k * 4, -16.0);
        put_f(&mut brush, offset("cbrush_t", "maxs") + k * 4, 16.0);
    }
    put(&mut brush, offset("cbrush_t", "contents"), 1);
    put(&mut brush, offset("cbrush_t", "numsides"), 1);
    put(&mut brush, offset("cbrush_t", "sides"), reference(side_at));
    z.virt_data(16, &brush);

    // MapEnts
    let ents = "{\n\"classname\" \"worldspawn\"\n}\n{\n\"classname\" \"mp_tdm_spawn\"\n\"origin\" \"1 2 3\"\n}\n";
    let mut m = image("MapEnts");
    put(&mut m, offset("MapEnts", "name"), INLINE);
    put(&mut m, offset("MapEnts", "entityString"), INLINE);
    put(&mut m, offset("MapEnts", "numEntityChars"), ents.len() as u32 + 1);
    z.temp(&m);
    z.string("maps/mp/mp_test.d3dbsp");
    z.string(ents);

    // GfxWorld: name, indices, then (reordered) vertices, then surfaces.
    let mut w = image("GfxWorld");
    put(&mut w, offset("GfxWorld", "name"), INLINE);
    put(&mut w, offset("GfxWorld", "indexCount"), 3);
    put(&mut w, offset("GfxWorld", "indices"), INLINE);
    put(&mut w, offset("GfxWorld", "surfaceCount"), 1);
    put(&mut w, offset("GfxWorld", "vertexCount"), 3);
    put(&mut w, offset("GfxWorld", "vd::vertices"), INLINE);
    put(&mut w, offset("GfxWorld", "dpvs::surfaces"), INLINE);
    put(&mut w, offset("GfxWorld", "dpvs::litSurfsEnd"), 1);
    put_f(&mut w, offset("GfxWorld", "sunParse::angles") + 4, 45.0);
    z.temp(&w);
    z.string("maps/mp/mp_test.d3dbsp");
    let mut idx = Vec::new();
    for i in [0u16, 1, 2] {
        idx.extend_from_slice(&i.to_le_bytes());
    }
    z.virt_data(2, &idx);
    let mut verts = Vec::new();
    for p in [[0.0f32, 0.0, 0.0], [64.0, 0.0, 0.0], [0.0, 64.0, 8.0]] {
        let mut v = image("GfxWorldVertex");
        for k in 0..3 {
            put_f(&mut v, k * 4, p[k]);
        }
        verts.extend(v);
    }
    z.virt_data(4, &verts);
    let mut surf = image("GfxSurface");
    put(&mut surf, offset("GfxSurface", "tris::vertexCount"), 3);
    surf[offset("GfxSurface", "tris::triCount")..][..2].copy_from_slice(&1u16.to_le_bytes());
    z.virt_data(4, &surf);
    z.bytes
}

#[test]
fn synthetic_map_converts() {
    let zone = Zone::parse(&synthetic_zone(), ParseOptions::default()).unwrap();
    assert_eq!(zone.stats.stopped_at, None);
    assert_eq!(zone.stats.top_level_parsed, 3);
    let map = to_iw3_map(&zone);
    assert!(map.notes.is_empty(), "{:?}", map.notes);
    let z = &map.zone;

    let clip = z.clip_map().expect("clipMap");
    assert_eq!(clip.name, "maps/mp/mp_test.d3dbsp");
    assert_eq!(clip.planes.len(), 2);
    assert_eq!(clip.planes[1].dist, 32.0);
    assert_eq!(clip.brushes.len(), 1);
    assert_eq!(clip.brushes[0].side_planes, vec![1]);
    assert_eq!(clip.brushes[0].maxs, [16.0; 3]);
    assert_eq!(clip.brushes[0].contents, 1);

    let ents = z.map_ents().expect("MapEnts");
    let parsed = iw3::ents::parse(&ents.entity_string);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[1].classname(), "mp_tdm_spawn");
    assert_eq!(parsed[1].origin(), Some([1.0, 2.0, 3.0]));

    let world = z.gfx_world().expect("GfxWorld");
    assert_eq!(world.indices, vec![0, 1, 2]);
    assert_eq!(world.vertices.len(), 3);
    assert_eq!(world.vertices[2].xyz, [0.0, 64.0, 8.0]);
    assert_eq!(world.surfaces.len(), 1);
    assert_eq!(world.surfaces[0].tri_count, 1);
    assert_eq!(world.surfaces[0].vertex_count, 3);
    assert_eq!(world.lit_surfs, 0..1);
    assert_eq!(world.sun.angles, [0.0, 45.0, 0.0]);
}

#[test]
fn truncated_zone_stops_with_a_message() {
    let full = synthetic_zone();
    for cut in [60, 200, 400, full.len() - 10] {
        let Ok(zone) = Zone::parse(&full[..cut], ParseOptions::default()) else {
            assert!(cut < 100, "cut {cut}: only a cut header is an error");
            continue;
        };
        let stop = zone.stats.stopped_at.as_deref().unwrap_or_default();
        assert!(stop.contains("past the end") || stop.contains("read past"), "cut {cut}: {stop}");
        // Converting a partial zone never panics.
        let _ = to_iw3_map(&zone);
    }
}

#[test]
fn corrupt_data_does_not_panic() {
    let clean = synthetic_zone();
    // Flip bytes after the asset list (headers, counts, pointers) in many
    // patterns; parsing and converting must fail or stop, never panic.
    let mut seed = 0x2545_f491_u32;
    for round in 0..400 {
        let mut data = clean.clone();
        for _ in 0..1 + round % 6 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let at = 36 + 16 + 24 + (seed >> 8) as usize % (data.len() - 76);
            data[at] = match seed % 4 {
                0 => 0xFF,
                1 => 0xFE,
                2 => (seed >> 16) as u8,
                _ => 0x7F,
            };
        }
        if let Ok(zone) = Zone::parse(&data, ParseOptions::default()) {
            let _ = to_iw3_map(&zone);
        }
    }
}

#[test]
fn unknown_asset_type_stops_cleanly() {
    let mut z = Stream::new(&[99]);
    z.temp(&[0; 16]);
    let zone = Zone::parse(&z.bytes, ParseOptions::default()).unwrap();
    assert!(zone.stats.stopped_at.unwrap().contains("invalid type 99"));
}
