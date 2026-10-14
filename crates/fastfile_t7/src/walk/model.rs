//! Models: `XModelMesh`, `XModel`, physics presets and constraints.

use super::capture::{LodRec, MeshRec, ModelRec, Skeleton, SurfaceRec};
use super::image::XPakEntry;
use super::simple::placeholder;
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::{PHYSICAL, Stream};

const SURFACE: usize = 0x60;
const SURFACE_SHARED: usize = 0x50;
const RIGID_VERT_LIST: usize = 16;
const COLLISION_TREE: usize = 0x38;

/// An `XSurfaceShared` (0x50, aligned 16) the field at `field` loads: the
/// struct, then its vertex/index data (`dataSize` bytes, PHYSICAL, aligned
/// 256: the PHYSICAL totals of `en_core_common` and `mp_patch`) unless
/// the data is streamed (`flags & 1`: in the `.xpak` entry the mesh's
/// `xpakEntry` names).
fn surface_shared(s: &mut Stream, field: usize, rec: Option<&mut MeshRec>) -> Result<()> {
    if let Some(sh) = s.array(field, SURFACE_SHARED, 15)? {
        s.no_loads(sh, SURFACE_SHARED, &[0x10])?;
        let flags = s.u32_at(sh)?;
        let size = s.u32_at(sh + 0x18)? as usize;
        let mut data = None;
        if flags & 1 == 0 {
            s.push(PHYSICAL);
            let r = s.array(sh + 0x10, size, SHARED_DATA_ALIGN);
            s.pop();
            data = r?.map(|at| (at, size));
        }
        if let Some(rec) = rec {
            rec.shared_flags = flags;
            rec.vert_count = s.u32_at(sh + 4)?;
            rec.skin_vert_count = s.u32_at(sh + 8)?;
            rec.index_count = s.u32_at(sh + 0xc)?;
            rec.data_size = size as u32;
            for (i, o) in [0x1c, 0x20, 0x24, 0x28].into_iter().enumerate() {
                rec.offsets[i] = s.u32_at(sh + o)?;
            }
            rec.data = data;
        }
    }
    Ok(())
}

const SHARED_DATA_ALIGN: u64 = 255;

/// `XModelMesh` (0x78; x64-zt layout): name, the shared vertex data, the
/// surfaces (0x60 each), then per surface its rigid vertex lists (16 each,
/// each with its collision tree: 0x38, nodes 16 each aligned 16, leaves 2
/// each) and its name. The surfaces' `shared` are references to the
/// mesh's. (Read on retail `mp_common`: its 697 leading meshes walk with
/// all 4800 references landing; whether the 16-alignment is on the nodes
/// or the leaves the references cannot tell.)
pub fn xmodel_mesh(s: &mut Stream, h: usize) -> Result<Option<String>> {
    // 0x0c..0x3c is the part bits (a bone mask: all-ones words are not
    // pointers); the pointers are the name, the surfaces and the shared data.
    s.no_loads(h + 0x40, AssetType::XModelMesh.header_size() - 0x40, &[0x28, 0x30])?;
    let name = s.string(h)?;
    let mut rec = MeshRec {
        name: name.clone().unwrap_or_else(String::new),
        ..Default::default()
    };
    let key = s.u64_at(h + 0x48)?;
    if key != 0 {
        rec.xpak = Some(XPakEntry {
            key,
            offset: s.u64_at(h + 0x50)? as i64,
            size: s.u64_at(h + 0x58)?,
            bits: s.u64_at(h + 0x60)?,
            width: 0,
            height: 0,
            levels: 0,
        });
    }
    surface_shared(s, h + 0x70, Some(&mut rec))?;
    let n = s.u8_at(h + 0x3c)? as usize;
    if let Some(t) = s.records(h + 0x68, n, SURFACE, 7)? {
        for i in 0..n {
            let surf = t + SURFACE * i;
            // (0x20..0x50 is the part bits: all-ones words are no pointers.)
            s.no_loads(surf, 0x20, &[0x10, 0x18])?;
            s.no_loads(surf + 0x50, 0x10, &[8])?;
            surface_shared(s, surf + 0x10, None)?;
            let lists = s.u8_at(surf + 1)? as usize;
            let mut rigid = Vec::new();
            if let Some(v) = s.records(surf + 0x18, lists, RIGID_VERT_LIST, 7)? {
                for k in 0..lists {
                    let list = v + RIGID_VERT_LIST * k;
                    rigid.push([s.u16_at(list)?, s.u16_at(list + 2)?, s.u16_at(list + 4)?, s.u16_at(list + 6)?]);
                    if let Some(tree) = s.array(list + 8, COLLISION_TREE, 7)? {
                        s.no_loads(tree, COLLISION_TREE, &[0x20, 0x30])?;
                        let nodes = s.u32_at(tree + 0x18)? as usize;
                        s.records(tree + 0x20, nodes, 16, 15)?;
                        let leaves = s.u32_at(tree + 0x28)? as usize;
                        s.records(tree + 0x30, leaves, 2, 1)?;
                    }
                }
            }
            let surf_name = s.string_named(surf + 0x58)?.unwrap_or_else(String::new);
            rec.surfaces.push(SurfaceRec {
                name: surf_name,
                vert_count: s.u16_at(surf + 4)?,
                tri_count: s.u16_at(surf + 6)?,
                base_vert: s.u32_at(surf + 8)?,
                base_index: s.u32_at(surf + 0xc)?,
                flags: s.u16_at(surf + 2)?,
                rigid,
            });
        }
    }
    s.capture.add_mesh(rec);
    Ok(name)
}

const MESH_MATERIALS: usize = 24;
const COLL_SURF: usize = 48;
const COLL_TRI: usize = 48;
const BONE_INFO: usize = 44;
const BASE_MAT: usize = 32;
const SUBMODEL: usize = 40;
const COLLMAP: usize = 16;
const GEOM_LIST: usize = 24;
const GEOM_INFO: usize = 72;

/// `XModel` (0x188; x64-zt layout). Stream order (read on retail
/// `mp_common`): name, bone names (u32 each), parent list (u8),
/// tag angles (8), tag positions (16, aligned 4) for the non-root bones,
/// part classification (u8) and base matrices (32, aligned 4) for all
/// bones, the LOD
/// meshes (assets; `-2` pointers, so each takes a VIRTUAL slot first), the
/// mesh materials (24 per LOD: `{ u16 count; Material** materials;
/// f32* himipInvSqRadii }`, then per LOD its material pointers, each
/// material, its radii), collision surfaces (48, each with its triangles,
/// 48 each, aligned 4), bone info (44 per bone), the collision LOD mesh (0x110, an
/// asset pointer, not x64-zt's `XModelMesh**`), the physics preset,
/// collmaps, physics constraints and submodels.
pub fn xmodel(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let mut known: Vec<usize> = vec![0, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xc8, 0xd8, 0xe8, 0x110, 0x128, 0x130, 0x138, 0x148];
    known.extend((0..8).map(|i| 0x88 + 8 * i));
    // (0x154.. is the combined part bits: all-ones words are no pointers.)
    s.no_loads(h, 0x150, &known)?;
    let name = s.string(h)?;
    let bones = s.u8_at(h + 8)? as usize + s.u16_at(h + 0xa)? as usize;
    let roots = s.u8_at(h + 9)?;
    let non_root = bones.saturating_sub(roots as usize);
    let names_at = s.records_at(h + 0x10, bones, 4, 3)?;
    let parents_at = s.records_at(h + 0x18, non_root, 1, 0)?;
    let quats_at = s.records_at(h + 0x20, non_root, 8, 1)?;
    let trans_at = s.records_at(h + 0x28, non_root, 16, 3)?;
    let parts_at = s.records_at(h + 0x30, bones, 1, 0)?;
    let mats_at = s.records_at(h + 0x38, bones, BASE_MAT, 3)?;
    let skeleton = skeleton(s, bones, roots, [names_at, parents_at, quats_at, trans_at, parts_at, mats_at])?;
    let lods = (s.u8_at(h + 0x40)? as usize).min(8);
    let mut rec = ModelRec {
        name: name.clone().unwrap_or_else(String::new),
        header_at: h,
        bones,
        skeleton,
        ..Default::default()
    };
    for i in 0..lods {
        let mesh = s
            .asset_named(AssetType::XModelMesh, h + 0x88 + 8 * i)?
            .unwrap_or_else(String::new);
        rec.lods.push(LodRec {
            mesh,
            materials: Vec::new(),
        });
    }
    if let Some(t) = s.records(h + 0xc8, lods, MESH_MATERIALS, 7)? {
        for i in 0..lods {
            let m = t + MESH_MATERIALS * i;
            let n = s.u16_at(m)? as usize;
            if let Some(list) = s.records(m + 8, n, 8, 7)? {
                for k in 0..n {
                    if let Some(mat) = s.asset_named(AssetType::Material, list + 8 * k)? {
                        rec.lods[i].materials.push(mat);
                    }
                }
            }
            s.records(m + 16, n, 4, 3)?;
        }
    }
    let surfs = s.i32_at(h + 0xe0)?.max(0) as usize;
    if let Some(t) = s.records(h + 0xd8, surfs, COLL_SURF, 7)? {
        for i in 0..surfs {
            let c = t + COLL_SURF * i;
            let tris = s.i32_at(c + 8)?.max(0) as usize;
            s.records(c, tris, COLL_TRI, 3)?;
        }
    }
    // Bone info for the model's own bones, not its cosmetic ones (0x0a;
    // `mp_nuketown_x`'s character models: 121 + 185).
    s.records(h + 0xe8, s.u8_at(h + 8)? as usize, BONE_INFO, 3)?;
    // The collision LOD: a mesh asset (`-2` like the LOD meshes).
    rec.collision_mesh = s.asset_named(AssetType::XModelMesh, h + 0x110)?;
    rec.phys_preset = s.asset_named(AssetType::PhysPreset, h + 0x128)?;
    let collmaps = s.u8_at(h + 0x118)? as usize;
    if let Some(t) = s.records(h + 0x130, collmaps, COLLMAP, 7)? {
        for i in 0..collmaps {
            if let Some(g) = s.array(t + COLLMAP * i, GEOM_LIST, 7)? {
                let n = s.u32_at(g)? as usize;
                if let Some(geoms) = s.records(g + 8, n, GEOM_INFO, 15)? {
                    for k in 0..n {
                        brush(s, geoms + GEOM_INFO * k)?;
                    }
                }
            }
        }
    }
    s.asset(AssetType::PhysConstraints, h + 0x138)?;
    let subs = s.i32_at(h + 0x140)?.max(0) as usize;
    if let Some(t) = s.records(h + 0x148, subs, SUBMODEL, 7)? {
        for i in 0..subs {
            s.asset(AssetType::XModel, t + SUBMODEL * i)?;
        }
    }
    s.capture.add_model(rec);
    Ok(name)
}

/// bo2mp: the skeleton's arrays read where the walk found them (names u32
/// script strings; parents u8; rest quaternions as four halves; rest
/// translations packed vec3s; part classification u8; bind poses `DObjAnimMat`, 32).
fn skeleton(s: &Stream, bones: usize, roots: u8, at: [Option<usize>; 6]) -> Result<Skeleton> {
    let non_root = bones.saturating_sub(roots as usize);
    let [names, parents, quats, trans, parts, mats] = at;
    let mut k = Skeleton { num_root_bones: roots, ..Default::default() };
    if let Some(a) = names {
        for i in 0..bones {
            k.bone_names.push(s.u32_at(a + 4 * i)?);
        }
    }
    if let Some(a) = parents {
        for i in 0..non_root {
            k.parent_list.push(s.u8_at(a + i)?);
        }
    }
    if let Some(a) = quats {
        for i in 0..non_root {
            let q = a + 8 * i;
            k.quats.push([0, 2, 4, 6].map(|o| s.u16_at(q + o).map_or(0.0, crate::mesh::f16)));
        }
    }
    // (The translations are packed vec3s, 12 bytes each, though the array
    // the stream holds is 16 a bone.)
    if let Some(a) = trans {
        for i in 0..non_root {
            let t = a + 12 * i;
            k.trans.push([s.f32_at(t)?, s.f32_at(t + 4)?, s.f32_at(t + 8)?]);
        }
    }
    if let Some(a) = parts {
        for i in 0..bones {
            k.part_classification.push(s.u8_at(a + i)?);
        }
    }
    if let Some(a) = mats {
        for i in 0..bones {
            let m = a + BASE_MAT * i;
            let mut v = [0f32; 8];
            for (j, x) in v.iter_mut().enumerate() {
                *x = s.f32_at(m + 4 * j)?;
            }
            k.base_mat.push(v);
        }
    }
    Ok(k)
}

const BRUSH: usize = 0x70;
const BRUSH_SIDE: usize = 16;
const PLANE: usize = 20;

/// A collmap geom's `BrushWrapper` (0x70, x64-zt layout: bounds, 0x1c u32
/// numsides, 0x20 cbrushside_t* sides (16: `{ cplane_t* plane; contents;
/// sflags }`), axial flags, 0x58 u32 numverts, 0x60 vec3* verts, 0x68
/// cplane_t* planes (20 each)). Order read on retail `mp_common`: the
/// struct (aligned 16; the geoms holding it too), the sides, each side's
/// plane (aligned 4), the verts, the planes
/// (usually a reference to the first side's plane).
fn brush(s: &mut Stream, field: usize) -> Result<()> {
    if let Some(b) = s.array(field, BRUSH, 15)? {
        s.no_loads(b, BRUSH, &[0x20, 0x60, 0x68])?;
        let sides = s.u32_at(b + 0x1c)? as usize;
        if let Some(t) = s.records(b + 0x20, sides, BRUSH_SIDE, 7)? {
            for i in 0..sides {
                s.array(t + BRUSH_SIDE * i, PLANE, 3)?;
            }
        }
        let verts = s.u32_at(b + 0x58)? as usize;
        s.records(b + 0x60, verts, 12, 3)?;
        s.records(b + 0x68, sides, PLANE, 3)?;
    }
    Ok(())
}

pub fn phys_constraints(s: &mut Stream, h: usize) -> Result<Option<String>> {
    placeholder(s, AssetType::PhysConstraints, h, 0)
}
