//! World at War assets as `iw3` ones.
//!
//! World at War is CoD4's engine one step on: images, the packed vertex,
//! the world vertex, rigid vertex lists and render state bits are laid out
//! exactly as in CoD4; models add physics/collision pointers, materials have
//! more techniques (fade and charred variants), and the world and collision
//! structs grow fields and reorder a few (see [`gfx_world`], [`clip_map`]).
//! Every field here is read by name through the schema generated from
//! OpenAssetTools' T4 definitions, never by a hand-written offset.
//!
//! [`to_iw3_map`] builds an `iw3::zone::Zone` holding a map's world,
//! collision, entities and what they use, so the CoD4 map lane's builders
//! take a World at War map unchanged. Images are streamed from the `.iwd`
//! archives by name, like CoD4's.
//!
//! Models, materials and images: from Nilbong/COD4-Rewrite's `t4::convert`
//! (MIT OR Apache-2.0). World, collision, entities: bo2mp.

use crate::zone::reader::{self, Ptr};
use crate::zone::schema::schema;
use crate::zone::{AssetType, GNode, GVal, Zone};
use iw3::zone as z3;

/// World at War technique index for each CoD4 one. World at War inserts
/// `emissive_reflected`, a `_fade` variant after each lit technique, the
/// `_charred` lit techniques and charred lights.
fn t4_technique(iw3_index: usize) -> Option<usize> {
    Some(match iw3_index {
        0..=6 => iw3_index,
        // lit, lit_sun, lit_sun_shadow, lit_spot, lit_spot_shadow, lit_omni, lit_omni_shadow
        7..=13 => 8 + (iw3_index - 7) * 2,
        // lit_instanced.. and light_spot, light_omni, light_spot_shadow
        14..=23 => iw3_index + 22,
        // fakelight_normal .. debug_bumpmap_instanced
        24..=33 => iw3_index + 25,
        _ => return None,
    })
}

/// The bytes of a loaded array, whether it came back as primitives or as
/// pointer-free records.
fn raw(n: &GNode, field: &str) -> Vec<u8> {
    match n.field(field) {
        Some(GVal::Bytes(b)) => b.clone(),
        Some(GVal::Nodes(nodes)) => nodes.iter().flat_map(|c| c.data.iter().copied()).collect(),
        _ => Vec::new(),
    }
}

fn u16s(b: &[u8]) -> impl Iterator<Item = u16> + '_ {
    b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]))
}

fn f32s(b: &[u8]) -> impl Iterator<Item = f32> + '_ {
    b.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap()))
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4).map_or(0, |c| u32::from_le_bytes(c.try_into().unwrap()))
}

/// A `vec3_t` member read from a struct image (the path reader indexes
/// whole members, not their elements).
fn vec3(n: &GNode, member: &str) -> [f32; 3] {
    let offset = schema().types[&n.ty].members.iter().find(|m| m.name.as_deref() == Some(member)).map_or(0, |m| m.offset as usize);
    std::array::from_fn(|k| f32::from_bits(u32_at(&n.data, offset + k * 4)))
}

fn i32s(b: &[u8]) -> impl Iterator<Item = i32> + '_ {
    b.chunks_exact(4).map(|c| i32::from_le_bytes(c.try_into().unwrap()))
}

/// Three consecutive floats of an array member (`origin`, `sunParse::angles`).
fn f3(n: &GNode, path: &str) -> [f32; 3] {
    std::array::from_fn(|k| n.float(&format!("{path}[{k}]")))
}

/// `count` consecutive floats of an array member, from element `first`.
fn fk<const N: usize>(n: &GNode, path: &str, first: usize) -> [f32; N] {
    std::array::from_fn(|k| n.float(&format!("{path}[{}]", first + k)))
}

/// Byte offset of a member of the node's own struct.
fn member_offset(n: &GNode, member: &str) -> Option<usize> {
    schema().types.get(&n.ty)?.members.iter().find(|m| m.name.as_deref() == Some(member)).map(|m| m.offset as usize)
}

/// A fixed `char[len]` member as a string.
fn cstr(n: &GNode, offset: usize, len: usize) -> String {
    let b = n.data.get(offset..(offset + len).min(n.data.len())).unwrap_or(&[]);
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

/// The raw pointer value of a pointer member.
fn ptr_of(n: &GNode, member: &str) -> Ptr {
    Ptr::from_raw(n.int(member) as u32)
}

/// Where an array member lives in the zone's blocks: where it was loaded
/// inline, or where its pointer points if it refers to data loaded earlier.
fn array_base(n: &GNode, member: &str) -> Option<(usize, u32)> {
    if let Some(k) = n.loc(member) {
        return Some(((k >> reader::OFFSET_BITS) as usize, k & ((1 << reader::OFFSET_BITS) - 1)));
    }
    match ptr_of(n, member) {
        Ptr::Ref { block, offset } => Some((block, offset)),
        _ => None,
    }
}

/// Index of the record a reference points at, in an array of `size` byte
/// records starting at `base`.
fn index_in(base: Option<(usize, u32)>, ptr: Ptr, size: u32) -> Option<u32> {
    let (bb, bo) = base?;
    match ptr {
        Ptr::Ref { block, offset } if block == bb && offset >= bo && (offset - bo) % size == 0 => Some((offset - bo) / size),
        _ => None,
    }
}

/// A World at War map zone as a CoD4 one, with notes on anything that did
/// not convert cleanly.
pub struct MapZone {
    pub zone: z3::Zone,
    pub notes: Vec<String>,
}

/// The map's world (GfxWorld), collision (clipMap_t), entities (MapEnts),
/// lights (ComWorld) and the models, materials, technique sets and images
/// they use, as an `iw3::zone::Zone`: the CoD4 map lane's builders take it
/// unchanged. Asset ids are renumbered; references between kept assets
/// follow, others become `None`.
pub fn to_iw3_map(zone: &Zone) -> MapZone {
    let kept = |ty: AssetType| {
        matches!(
            ty,
            AssetType::XModel
                | AssetType::Material
                | AssetType::TechniqueSet
                | AssetType::Image
                | AssetType::MapEnts
                | AssetType::GfxWorld
                | AssetType::ClipMap
                | AssetType::ClipMapPvs
                | AssetType::ComWorld
                | AssetType::GameWorldMp
                // bo2mp: the map's scripts as text (its createart fog).
                | AssetType::RawFile
        )
    };
    let mut map = vec![None; zone.assets.len()];
    let mut order = Vec::new();
    for (i, a) in zone.assets.iter().enumerate() {
        if kept(a.ty) {
            map[i] = Some(order.len());
            order.push(i);
        }
    }
    let remap = |id: Option<usize>| id.and_then(|i| map.get(i).copied().flatten());
    let mut notes = Vec::new();
    let assets = order
        .iter()
        .map(|&i| {
            let a = &zone.assets[i];
            match a.ty {
                AssetType::XModel => z3::Asset::XModel(xmodel(zone, &a.root, &remap)),
                AssetType::Material => z3::Asset::Material(material(&a.root, &remap)),
                AssetType::TechniqueSet => z3::Asset::TechniqueSet(z3::TechniqueSet {
                    name: a.name.clone(),
                    world_vert_format: a.root.int("worldVertFormat") as u8,
                    techniques: Vec::new(),
                }),
                AssetType::MapEnts => z3::Asset::MapEnts(map_ents(&a.root)),
                AssetType::GfxWorld => z3::Asset::GfxWorld(Box::new(gfx_world(&a.root, &remap))),
                AssetType::ClipMap | AssetType::ClipMapPvs => {
                    let (clip, mut n) = clip_map(&a.root, &remap);
                    notes.append(&mut n);
                    z3::Asset::ClipMap(Box::new(clip))
                }
                AssetType::ComWorld => z3::Asset::ComWorld(com_world(&a.root)),
                AssetType::GameWorldMp => z3::Asset::GameWorldMp(a.name.clone()),
                AssetType::RawFile => z3::Asset::RawFile(raw_file(&a.root, &a.name)),
                _ => z3::Asset::Image(image(&a.root)),
            }
        })
        .collect();
    let stats = z3::ZoneStats {
        top_level_total: zone.stats.top_level_total,
        top_level_parsed: zone.stats.top_level_parsed,
        stopped_at: zone.stats.stopped_at.clone(),
        unresolved_aliases: zone.stats.unresolved_aliases,
        unresolved_strings: zone.stats.unresolved_strings,
        block_positions: Default::default(),
    };
    let top_level = zone.top_level.iter().map(|&(ty, id)| (ty, remap(id))).collect();
    MapZone { zone: z3::Zone { script_strings: zone.script_strings.clone(), assets, top_level, stats }, notes }
}

/// bo2mp: a rawfile (`name`, `len`, `buffer`): its text up to `len`.
fn raw_file(r: &GNode, asset_name: &str) -> z3::RawFile {
    let name = r.string("name").map_or_else(|| asset_name.to_owned(), str::to_owned);
    let len = r.int("len").max(0) as usize;
    let mut data = match r.field("buffer") {
        Some(GVal::Str(Some(s))) => s.clone().into_bytes(),
        _ => raw(r, "buffer"),
    };
    if len > 0 && data.len() > len {
        data.truncate(len);
    }
    z3::RawFile { name, data }
}

pub fn map_ents(m: &GNode) -> z3::MapEnts {
    // `entityString` is loaded as a counted char array (no `set string`).
    let entity_string = match m.field("entityString") {
        Some(GVal::Str(Some(s))) => s.clone(),
        _ => {
            let b = raw(m, "entityString");
            let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
            String::from_utf8_lossy(&b[..end]).into_owned()
        }
    };
    z3::MapEnts { name: m.string("name").unwrap_or("").to_owned(), entity_string }
}

pub fn com_world(c: &GNode) -> z3::ComWorld {
    z3::ComWorld {
        name: c.string("name").unwrap_or("").to_owned(),
        primary_lights: c
            .nodes("primaryLights")
            .iter()
            .map(|l| z3::PrimaryLight {
                kind: l.int("type") as u8,
                color: f3(l, "color"),
                dir: f3(l, "dir"),
                origin: f3(l, "origin"),
                radius: l.float("radius"),
                cos_half_fov_outer: l.float("cosHalfFovOuter"),
                cos_half_fov_inner: l.float("cosHalfFovInner"),
                def_name: l.string("defName").map(str::to_owned),
            })
            .collect(),
    }
}

/// World at War's `GfxWorld` (800 bytes): CoD4's with vertex layers,
/// coronas, a world LOD chain and water; its surfaces carry a
/// `srfTriangles_t` (and a pad) before the material, and its static model
/// instances are 92 bytes.
pub fn gfx_world(w: &GNode, remap: &dyn Fn(Option<usize>) -> Option<usize>) -> z3::GfxWorld {
    let vertices = w
        .node("vd")
        .map(|vd| raw(vd, "vertices"))
        .unwrap_or(Vec::new())
        .chunks_exact(44)
        .map(|v| z3::WorldVertex {
            xyz: std::array::from_fn(|k| f32::from_bits(u32_at(v, k * 4))),
            binormal_sign: f32::from_bits(u32_at(v, 12)),
            color: u32_at(v, 16),
            tex_coord: [f32::from_bits(u32_at(v, 20)), f32::from_bits(u32_at(v, 24))],
            lmap_coord: [f32::from_bits(u32_at(v, 28)), f32::from_bits(u32_at(v, 32))],
            normal: u32_at(v, 36),
            tangent: u32_at(v, 40),
        })
        .collect();
    let dpvs = w.node("dpvs");
    let surfaces = dpvs
        .map(|d| d.nodes("surfaces"))
        .unwrap_or(&[])
        .iter()
        .map(|s| z3::GfxSurface {
            first_vertex: s.int("tris::firstVertex") as i32,
            vertex_count: s.int("tris::vertexCount") as u16,
            tri_count: s.int("tris::triCount") as u16,
            base_index: s.int("tris::baseIndex") as i32,
            material: remap(s.asset("material")),
            lightmap_index: s.int("lightmapIndex") as u8,
            reflection_probe_index: s.int("reflectionProbeIndex") as u8,
            primary_light_index: s.int("primaryLightIndex") as u8,
            flags: s.int("flags") as u8,
            bounds: [fk(s, "bounds", 0), fk(s, "bounds", 3)],
        })
        .collect();
    let static_models = dpvs
        .map(|d| d.nodes("smodelDrawInsts"))
        .unwrap_or(&[])
        .iter()
        .map(|m| z3::StaticModel {
            model: remap(m.asset("model")),
            origin: f3(m, "placement::origin"),
            axis: [fk(m, "placement::axis", 0), fk(m, "placement::axis", 3), fk(m, "placement::axis", 6)],
            scale: m.float("placement::scale"),
            cull_dist: m.float("cullDist"),
            flags: m.int("flags") as u8,
        })
        .collect();
    let sun_parse = member_offset(w, "sunParse").unwrap_or(0);
    let sun = z3::SunParse {
        name: cstr(w, sun_parse, 64),
        ambient_scale: w.float("sunParse::ambientScale"),
        ambient_color: f3(w, "sunParse::ambientColor"),
        diffuse_fraction: w.float("sunParse::diffuseFraction"),
        sun_light: w.float("sunParse::sunLight"),
        sun_color: f3(w, "sunParse::sunColor"),
        diffuse_color: f3(w, "sunParse::diffuseColor"),
        angles: f3(w, "sunParse::angles"),
    };
    let sun_flare = w
        .node("sun")
        .map(|f| z3::SunFlare {
            valid: f.int("hasValidData") != 0,
            sprite: remap(f.asset("spriteMaterial")),
            flare: remap(f.asset("flareMaterial")),
            sprite_size: f.float("spriteSize"),
            flare_min_size: f.float("flareMinSize"),
            flare_min_dot: f.float("flareMinDot"),
            flare_max_size: f.float("flareMaxSize"),
            flare_max_dot: f.float("flareMaxDot"),
            flare_max_alpha: f.float("flareMaxAlpha"),
            flare_fade_in: f.int("flareFadeInTime") as i32,
            flare_fade_out: f.int("flareFadeOutTime") as i32,
            blind_min_dot: f.float("blindMinDot"),
            blind_max_dot: f.float("blindMaxDot"),
            blind_max_darken: f.float("blindMaxDarken"),
            blind_fade_in: f.int("blindFadeInTime") as i32,
            blind_fade_out: f.int("blindFadeOutTime") as i32,
            glare_min_dot: f.float("glareMinDot"),
            glare_max_dot: f.float("glareMaxDot"),
            glare_max_lighten: f.float("glareMaxLighten"),
            glare_fade_in: f.int("glareFadeInTime") as i32,
            glare_fade_out: f.int("glareFadeOutTime") as i32,
            fx_position: f3(f, "sunFxPosition"),
        })
        .unwrap_or(z3::SunFlare::default());
    let lightmaps = w
        .nodes("lightmaps")
        .iter()
        .map(|l| z3::LightmapPair { primary: remap(l.asset("primary")), secondary: remap(l.asset("secondary")) })
        .collect();
    let reflection_probes = w
        .nodes("reflectionProbes")
        .iter()
        .map(|p| z3::ReflectionProbe { origin: f3(p, "origin"), image: remap(p.asset("reflectionImage")) })
        .collect();
    let models = w
        .nodes("models")
        .iter()
        .map(|m| z3::GfxBrushModel {
            bounds: [fk(m, "bounds", 0), fk(m, "bounds", 3)],
            surface_count: m.int("surfaceCount").clamp(0, u16::MAX as i64) as u16,
            start_surf_index: m.int("startSurfIndex").clamp(0, u16::MAX as i64) as u16,
        })
        .collect();
    let light_grid = match w.node("lightGrid") {
        Some(g) => z3::LightGrid {
            mins: std::array::from_fn(|k| g.int(&format!("mins[{k}]")) as u16),
            maxs: std::array::from_fn(|k| g.int(&format!("maxs[{k}]")) as u16),
            row_axis: g.int("rowAxis") as u32,
            col_axis: g.int("colAxis") as u32,
            sun_primary_light_index: g.int("sunPrimaryLightIndex") as u32,
            has_light_regions: g.int("hasLightRegions") != 0,
            row_data_start: u16s(&raw(g, "rowDataStart")).collect(),
            raw_row_data: raw(g, "rawRowData"),
            entries: raw(g, "entries")
                .chunks_exact(4)
                .map(|e| z3::LightGridEntry { colors_index: u16::from_le_bytes([e[0], e[1]]), primary_light_index: e[2], needs_trace: e[3] })
                .collect(),
            colors: raw(g, "colors")
                .chunks_exact(168)
                .map(|c| z3::LightGridColors(std::array::from_fn(|d| std::array::from_fn(|k| c[d * 3 + k]))))
                .collect(),
        },
        None => z3::LightGrid::default(),
    };
    let range = |begin: &str, end: &str| w.int(&format!("dpvs::{begin}")) as u32..w.int(&format!("dpvs::{end}")) as u32;
    z3::GfxWorld {
        name: w.string("name").unwrap_or("").to_owned(),
        base_name: w.string("baseName").unwrap_or("").to_owned(),
        indices: u16s(&raw(w, "indices")).collect(),
        vertices,
        surfaces,
        static_models,
        sky_start_surfs: i32s(&raw(w, "skyStartSurfs")).collect(),
        sky_image: remap(w.asset("skyImage")),
        sun,
        sun_flare,
        sun_color_from_bsp: f3(w, "sunColorFromBsp"),
        lightmaps,
        reflection_probes,
        mins: f3(w, "mins"),
        maxs: f3(w, "maxs"),
        lit_surfs: range("litSurfsBegin", "litSurfsEnd"),
        decal_surfs: range("decalSurfsBegin", "decalSurfsEnd"),
        emissive_surfs: range("emissiveSurfsBegin", "emissiveSurfsEnd"),
        models,
        light_grid,
    }
}

const PLANE_SIZE: u32 = 20;
const BRUSH_SIDE_SIZE: u32 = 12;

/// World at War's `clipMap_t` (332 bytes): CoD4's with brush vertices,
/// `uinds`, four dynamic entity lists and physics constraints. Its
/// `CollisionAabbTree` keeps the material and child count before the half
/// size (CoD4 has them after), and partitions are 20 bytes.
pub fn clip_map(c: &GNode, remap: &dyn Fn(Option<usize>) -> Option<usize>) -> (z3::ClipMap, Vec<String>) {
    let mut notes = Vec::new();
    let planes: Vec<z3::Plane> = raw(c, "planes")
        .chunks_exact(PLANE_SIZE as usize)
        .map(|p| z3::Plane {
            normal: std::array::from_fn(|k| f32::from_bits(u32_at(p, k * 4))),
            dist: f32::from_bits(u32_at(p, 12)),
        })
        .collect();
    let planes_base = array_base(c, "planes");
    let static_models = c
        .nodes("staticModelList")
        .iter()
        .map(|m| z3::ClipStaticModel {
            model: remap(m.asset("xmodel")),
            origin: f3(m, "origin"),
            inv_scaled_axis: [fk(m, "invScaledAxis", 0), fk(m, "invScaledAxis", 3), fk(m, "invScaledAxis", 6)],
            absmin: f3(m, "absmin"),
            absmax: f3(m, "absmax"),
        })
        .collect();
    let materials = c
        .nodes("materials")
        .iter()
        .map(|m| z3::ClipMaterial {
            name: cstr(m, 0, 64),
            surface_flags: m.int("surfaceFlags") as i32,
            content_flags: m.int("contentFlags") as i32,
        })
        .collect();

    // Brush sides: (plane index, material) each.
    let side_of = |s: &GNode| (index_in(planes_base, ptr_of(s, "plane"), PLANE_SIZE), s.int("materialNum") as u32);
    let sides: Vec<(Option<u32>, u32)> = c.nodes("brushsides").iter().map(side_of).collect();
    let mut sides_base = array_base(c, "brushsides");
    let brush_nodes = c.nodes("brushes");
    if sides_base.is_none() {
        // Sides are stored in brush order, so the lowest side reference is
        // the start of the array.
        sides_base = brush_nodes
            .iter()
            .filter_map(|b| match ptr_of(b, "sides") {
                Ptr::Ref { block, offset } => Some((block, offset)),
                _ => None,
            })
            .min_by_key(|&(_, o)| o);
    }
    let (mut unresolved_sides, mut unresolved_planes) = (0usize, 0usize);
    let mut brushes = Vec::with_capacity(brush_nodes.len());
    for b in brush_nodes {
        let n = b.int("numsides").max(0) as usize;
        // Sides inline under the brush (not expected) or shared with the
        // brush side array.
        let inline: Vec<(Option<u32>, u32)> = b.nodes("sides").iter().map(side_of).collect();
        let mine: Vec<(Option<u32>, u32)> = if !inline.is_empty() {
            inline
        } else if n == 0 {
            Vec::new()
        } else {
            match index_in(sides_base, ptr_of(b, "sides"), BRUSH_SIDE_SIZE).map(|i| i as usize) {
                Some(first) if first.saturating_add(n) <= sides.len() => sides[first..first + n].to_vec(),
                _ => {
                    unresolved_sides += 1;
                    Vec::new()
                }
            }
        };
        let mut side_planes = Vec::with_capacity(mine.len());
        let mut side_materials = Vec::with_capacity(mine.len());
        for (plane, material) in mine {
            match plane.filter(|&p| (p as usize) < planes.len()) {
                Some(p) => {
                    side_planes.push(p);
                    side_materials.push(material);
                }
                None => unresolved_planes += 1,
            }
        }
        brushes.push(z3::Brush {
            mins: f3(b, "mins"),
            maxs: f3(b, "maxs"),
            contents: b.int("contents") as i32,
            side_planes,
            side_materials,
            axial_materials: [
                std::array::from_fn(|k| b.int(&format!("axialMaterialNum[{k}]")) as i16),
                std::array::from_fn(|k| b.int(&format!("axialMaterialNum[{}]", 3 + k)) as i16),
            ],
        });
    }
    if unresolved_sides > 0 || unresolved_planes > 0 {
        notes.push(format!(
            "t4 clip: {unresolved_sides} brushes' side lists and {unresolved_planes} side planes could not be resolved \
             (those brushes keep only their bounding box)"
        ));
    }

    let leaf_brush_nodes = c
        .nodes("leafbrushNodes")
        .iter()
        .map(|n| {
            let count = n.int("leafBrushCount") as i16;
            let brushes = if count > 0 {
                n.node("data").and_then(|d| d.node("leaf")).map(|l| u16s(&raw(l, "brushes")).collect()).unwrap_or(Vec::new())
            } else {
                Vec::new()
            };
            let child_offsets = if count > 0 {
                [0; 2]
            } else {
                [n.int("data::children::childOffset[0]") as u16, n.int("data::children::childOffset[1]") as u16]
            };
            z3::LeafBrushNode { leaf_brush_count: count, brushes, child_offsets }
        })
        .collect();
    let partitions = c
        .nodes("partitions")
        .iter()
        .map(|p| z3::CollisionPartition { tri_count: p.int("triCount") as u8, first_tri: p.int("firstTri") as i32 })
        .collect();
    let aabb_trees = c
        .nodes("aabbTrees")
        .iter()
        .map(|t| z3::CollisionAabbTree {
            origin: f3(t, "origin"),
            half_size: f3(t, "halfSize"),
            material_index: t.int("materialIndex") as u16,
            child_count: t.int("childCount") as u16,
            index: t.int("u::firstChildIndex") as i32,
        })
        .collect();
    let cmodels = c
        .nodes("cmodels")
        .iter()
        .map(|m| z3::CModel {
            mins: f3(m, "mins"),
            maxs: f3(m, "maxs"),
            radius: m.float("radius"),
            leaf_brush_node: m.int("leaf::leafBrushNode") as i32,
            first_coll_aabb: m.int("leaf::firstCollAabbIndex") as u16,
            coll_aabb_count: m.int("leaf::collAabbCount") as u16,
        })
        .collect();
    let dyn_ent_counts = [c.int("dynEntCount[0]") as u16, c.int("dynEntCount[1]") as u16];
    let dyn_ents = c
        .nodes("dynEntDefList")
        .iter()
        .map(|d| z3::DynEntDef {
            kind: d.int("type") as i32,
            quat: fk(d, "pose::quat", 0),
            origin: f3(d, "pose::origin"),
            model: remap(d.asset("xModel")),
            brush_model: d.int("brushModel") as u16,
            physics_brush_model: d.int("physicsBrushModel") as u16,
            destroy_fx: None,
            destroy_pieces: None,
            phys_preset: None,
            health: d.int("health") as i32,
            contents: d.int("contents") as i32,
        })
        .collect();
    let clip = z3::ClipMap {
        name: c.string("name").unwrap_or("").to_owned(),
        planes,
        static_models,
        materials,
        brushes,
        verts: f32s(&raw(c, "verts")).collect::<Vec<_>>().chunks_exact(3).map(|v| [v[0], v[1], v[2]]).collect(),
        tri_indices: u16s(&raw(c, "triIndices")).collect(),
        partitions,
        aabb_trees,
        cmodels,
        map_ents: remap(c.asset("mapEnts")),
        dyn_ent_counts,
        dyn_ents,
        leaf_brush_nodes,
    };
    (clip, notes)
}

pub fn xmodel(zone: &Zone, m: &GNode, remap: &dyn Fn(Option<usize>) -> Option<usize>) -> z3::XModel {
    let quats = raw(m, "quats").chunks_exact(8).map(|q| std::array::from_fn(|k| i16::from_le_bytes([q[k * 2], q[k * 2 + 1]]))).collect();
    // Packed vec3s, though the allocation is sized for vec4s.
    let non_root = (m.int("numBones") - m.int("numRootBones")).max(0) as usize;
    let trans: Vec<f32> = f32s(&raw(m, "trans")).collect();
    let trans = trans.chunks_exact(3).take(non_root).map(|t| [t[0], t[1], t[2]]).collect();
    let base_mat = raw(m, "baseMat")
        .chunks_exact(32)
        .map(|b| {
            let f: Vec<f32> = f32s(b).collect();
            z3::BoneMat { quat: [f[0], f[1], f[2], f[3]], trans: [f[4], f[5], f[6]] }
        })
        .collect();
    let lods = (0..m.int("numLods").clamp(0, 4))
        .map(|i| z3::XModelLod {
            dist: m.float(&format!("lodInfo[{i}]::dist")),
            num_surfs: m.int(&format!("lodInfo[{i}]::numsurfs")) as u16,
            surf_index: m.int(&format!("lodInfo[{i}]::surfIndex")) as u16,
        })
        .collect();
    z3::XModel {
        name: m.string("name").unwrap_or("").to_owned(),
        num_bones: m.int("numBones") as u8,
        num_root_bones: m.int("numRootBones") as u8,
        bone_names: u16s(&raw(m, "boneNames")).map(|i| zone.script_strings.get(i as usize).cloned().unwrap_or(String::new())).collect(),
        parent_list: raw(m, "parentList"),
        quats,
        trans,
        base_mat,
        surfs: m.nodes("surfs").iter().map(surface).collect(),
        materials: m.assets("materialHandles").into_iter().map(remap).collect(),
        lods,
        radius: m.float("radius"),
        mins: vec3(m, "mins"),
        maxs: vec3(m, "maxs"),
        contents: m.int("contents") as i32,
        coll_boxes: Vec::new(),
    }
}

fn surface(s: &GNode) -> z3::XSurface {
    let info = s.node("vertInfo");
    let blend_counts = std::array::from_fn(|k| info.map_or(0, |i| i.int(&format!("vertCount[{k}]")) as i16));
    let verts = raw(s, "verts0")
        .chunks_exact(32)
        .map(|v| z3::PackedVertex {
            xyz: std::array::from_fn(|k| f32::from_bits(u32_at(v, k * 4))),
            binormal_sign: f32::from_bits(u32_at(v, 12)),
            color: u32_at(v, 16),
            tex_coord: u32_at(v, 20),
            normal: u32_at(v, 24),
            tangent: u32_at(v, 28),
        })
        .collect();
    let tris = u16s(&raw(s, "triIndices")).collect::<Vec<_>>().chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
    let vert_lists = s
        .nodes("vertList")
        .iter()
        .map(|l| z3::RigidVertList {
            bone_offset: l.int("boneOffset") as u16,
            vert_count: l.int("vertCount") as u16,
            tri_offset: l.int("triOffset") as u16,
            tri_count: l.int("triCount") as u16,
        })
        .collect();
    z3::XSurface {
        tile_mode: s.int("tileMode") as u8,
        deformed: s.int("deformed") != 0,
        vert_count: s.int("vertCount") as u16,
        tri_count: s.int("triCount") as u16,
        base_tri_index: s.int("baseTriIndex") as u16,
        base_vert_index: s.int("baseVertIndex") as u16,
        blend_counts,
        verts_blend: info.map(|i| u16s(&raw(i, "vertsBlend")).collect()).unwrap_or(Vec::new()),
        verts,
        tris,
        vert_lists,
    }
}

pub fn material(m: &GNode, remap: &dyn Fn(Option<usize>) -> Option<usize>) -> z3::Material {
    let info = m.node("info");
    let entries: Vec<u8> = (0..59).map(|i| m.int(&format!("stateBitsEntry[{i}]")) as u8).collect();
    let state_bits_entry = std::array::from_fn(|i| t4_technique(i).and_then(|t| entries.get(t).copied()).unwrap_or(0xff));
    let textures = m
        .nodes("textureTable")
        .iter()
        .map(|t| z3::MaterialTexture {
            name_hash: t.int("nameHash") as u32,
            name_start: t.int("nameStart") as u8,
            name_end: t.int("nameEnd") as u8,
            sampler_state: t.data.get(6).copied().unwrap_or(0),
            semantic: z3::TextureSemantic::from_u8(t.int("semantic") as u8),
            image: remap(t.node("u").and_then(|u| u.asset("image"))),
        })
        .collect();
    let constants = raw(m, "constantTable")
        .chunks_exact(32)
        .map(|c| z3::MaterialConstant {
            name_hash: u32_at(c, 0),
            name: String::from_utf8_lossy(&c[4..16]).trim_end_matches('\0').to_owned(),
            literal: std::array::from_fn(|k| f32::from_bits(u32_at(c, 16 + k * 4))),
        })
        .collect();
    let state_bits = raw(m, "stateBitsTable").chunks_exact(8).map(|b| [u32_at(b, 0), u32_at(b, 4)]).collect();
    z3::Material {
        name: info.and_then(|i| i.string("name")).unwrap_or("").to_owned(),
        game_flags: info.map_or(0, |i| i.int("gameFlags") as u8),
        sort_key: info.map_or(0, |i| i.int("sortKey") as u8),
        surface_type_bits: info.map_or(0, |i| i.int("surfaceTypeBits") as u32),
        state_bits_entry,
        state_flags: m.int("stateFlags") as u8,
        camera_region: m.int("cameraRegion") as u8,
        technique_set: remap(m.asset("techniqueSet")),
        textures,
        constants,
        state_bits,
    }
}

pub fn image(i: &GNode) -> z3::Image {
    let (width, height, depth) = (i.int("width") as u16, i.int("height") as u16, i.int("depth") as u16);
    let load_def = i.node("texture").and_then(|t| t.node("loadDef")).map(|l| z3::ImageLoadDef {
        level_count: l.int("levelCount") as u8,
        flags: l.int("flags") as u8,
        dimensions: [width, height, depth],
        format: l.int("format") as u32,
        data: l.data.get(16..).unwrap_or(&[]).to_vec(),
    });
    z3::Image {
        name: i.string("name").unwrap_or("").to_owned(),
        map_type: match i.int("mapType") {
            3 => z3::MapType::TwoD,
            4 => z3::MapType::ThreeD,
            5 => z3::MapType::Cube,
            o => z3::MapType::Other(o as u32),
        },
        semantic: i.int("semantic") as u8,
        category: i.int("category") as u8,
        width,
        height,
        depth,
        load_def,
    }
}

