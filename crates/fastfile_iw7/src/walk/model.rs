//! Models and their surfaces (x64-zt `iw7/assets/{xmodel,xsurface}.cpp`).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::{CALLBACK, VIRTUAL};
use crate::walk::{LodRec, ModelSurfsRec, Span, SurfaceRec, XModelRec};

pub(super) fn xmodel(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::XModel as X;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + X::name)?.unwrap_or_default();
    let num_bones = w.s.u8_at(h + X::numBones)?;
    let num_root = w.s.u8_at(h + X::numRootBones)?;
    let client_bones = w.s.u16_at(h + X::numClientBones)? as usize;
    let numsurfs = w.s.u16_at(h + X::numsurfs)?;
    let nb = num_bones as usize;
    let all = nb + client_bones;
    let nonroot = all.saturating_sub(num_root as usize);
    let mut rec = XModelRec {
        name,
        num_bones,
        num_root_bones: num_root,
        numsurfs: numsurfs as u8,
        scale: w.s.f32_at(h + X::scale)?,
        ..Default::default()
    };
    if let Some(at) = w.blob(h + X::boneNames, 4 * all, 3)? {
        rec.bone_names = (0..all).map(|i| w.s.u32_at(at + 4 * i)).collect::<Result<_>>()?;
    }
    if let Some(at) = w.blob(h + X::parentList, nonroot, 0)? {
        rec.parent_list = w.s.slice(at, nonroot)?.to_vec();
    }
    if let Some(at) = w.blob(h + X::tagAngles, 8 * nonroot, 1)? {
        rec.tag_angles = (0..nonroot)
            .map(|i| {
                let b = at + 8 * i;
                Ok([w.s.u16_at(b)? as i16, w.s.u16_at(b + 2)? as i16, w.s.u16_at(b + 4)? as i16, w.s.u16_at(b + 6)? as i16])
            })
            .collect::<Result<_>>()?;
    }
    if let Some(at) = w.blob(h + X::tagPositions, 12 * nonroot, 3)? {
        rec.tag_positions =
            (0..nonroot).map(|i| Ok([w.s.f32_at(at + 12 * i)?, w.s.f32_at(at + 12 * i + 4)?, w.s.f32_at(at + 12 * i + 8)?])).collect::<Result<_>>()?;
    }
    if let Some(at) = w.blob(h + X::partClassification, nb, 0)? {
        rec.part_classification = w.s.slice(at, nb)?.to_vec();
    }
    let mat = L::DObjAnimMat::SIZE;
    if let Some(at) = w.blob(h + X::baseMat, mat * all, 15)? {
        rec.base_mat = (0..all)
            .map(|i| {
                let mut m = [0f32; 8];
                for (k, v) in m.iter_mut().enumerate() {
                    *v = w.s.f32_at(at + mat * i + 4 * k)?;
                }
                Ok(m)
            })
            .collect::<Result<_>>()?;
    }
    let rmp = w.s.u8_at(h + X::numReactiveMotionParts)? as usize;
    w.blob(h + X::reactiveMotionParts, L::ReactiveMotionModelPart::SIZE * rmp, 15)?;
    w.blob(h + X::reactiveMotionTweaks, L::ReactiveMotionModelTweaks::SIZE, 15)?;
    w.s.inc(CALLBACK, 4 * numsurfs as u64);
    let ns = numsurfs as usize;
    if let Some(at) = w.inline_array(h + X::materialHandles, 8 * ns, 7)? {
        for i in 0..ns {
            let m = w.asset(AssetType::Material, at + 8 * i)?;
            rec.materials.push(m);
        }
    } else if let Some(v) = w.s.ptr_at(h + X::materialHandles)?.virtual_offset()
        && let Some(at) = w.vmap.file_pos(v)
    {
        for i in 0..ns {
            let m = w.s.ptr_at(at + 8 * i)?.virtual_offset().and_then(|v| w.resolve_slot(AssetType::Material, v));
            rec.materials.push(m);
        }
    }
    let li = L::XModelLodInfo::SIZE;
    for i in 0..6 {
        let l = h + X::lodInfo + li * i;
        let surfs = w.asset(AssetType::XModelSurfs, l + L::XModelLodInfo::modelSurfs)?;
        rec.lods.push(LodRec {
            dist: w.s.f32_at(l)?,
            numsurfs: w.s.u16_at(l + L::XModelLodInfo::numsurfs)?,
            surf_index: w.s.u16_at(l + L::XModelLodInfo::surfIndex)?,
            surfs,
        });
    }
    let ncoll = w.s.u32_at(h + X::numCollSurfs)? as usize;
    w.blob(h + X::collSurfs, L::XModelCollSurf_s::SIZE * ncoll, 3)?;
    w.blob(h + X::boneInfo, L::XBoneInfo::SIZE * all, 3)?;
    w.blob(h + X::invHighMipRadius, 2 * ns, 1)?;
    w.asset(AssetType::PhysicsAsset, h + X::physicsAsset)?;
    w.asset(AssetType::PhysicsFxShape, h + X::physicsFXShape)?;
    let lod_data = w.s.u32_at(h + X::physicsLODDataSize)? as usize;
    w.blob(h + X::physicsLODData, lod_data, 15)?;
    let lod_names = w.s.u32_at(h + X::physicsLODDataNameCount)? as usize;
    w.blob(h + X::physicsLODDataNames, 4 * lod_names, 3)?;
    w.asset(AssetType::Scriptable, h + X::scriptableMoverDef)?;
    w.asset(AssetType::XAnimProceduralBones, h + X::proceduralBones)?;
    let aim = w.s.u8_at(h + X::numAimAssistBones)? as usize;
    w.blob(h + X::aimAssistBones, 4 * aim, 3)?;
    let n3 = w.s.u8_at(h + X::unknown03Count)? as usize;
    w.blob(h + X::unknown03, n3, 0)?;
    let nv = w.s.u8_at(h + X::unknownVec3Count)? as usize;
    w.blob(h + X::unknownVec3, 12 * nv, 3)?;
    let n4 = w.s.u8_at(h + X::unknown04Count)? as usize;
    w.blob(h + X::unknown04, 24 * n4, 3)?;
    w.s.pop();
    rec.num_lods = w.s.u8_at(h + X::numLods)?;
    rec.coll_lod = w.s.u8_at(h + X::collLod)? as i8;
    rec.flags = w.s.u32_at(h + X::flags)? as u8;
    rec.contents = w.s.i32_at(h + X::contents)?;
    rec.radius = w.s.f32_at(h + X::radius)?;
    let b = h + X::bounds;
    rec.bounds = ([w.s.f32_at(b)?, w.s.f32_at(b + 4)?, w.s.f32_at(b + 8)?], [w.s.f32_at(b + 12)?, w.s.f32_at(b + 16)?, w.s.f32_at(b + 20)?]);
    w.cap.xmodels.push(rec);
    Ok(AssetId { ty: AssetType::XModel, index: (w.cap.xmodels.len() - 1) as u32 })
}

fn surface(w: &mut Walker, s: usize) -> Result<SurfaceRec> {
    use L::XSurface as X;
    let flags = w.s.u16_at(s + X::flags)?;
    let vert_count = w.s.u16_at(s + X::vertCount)?;
    let tri_count = w.s.u16_at(s + X::triCount)?;
    let rigid_n = w.s.u8_at(s + X::rigidVertListCount)? as usize;
    let subdiv_n = w.s.u8_at(s + X::subdivLevelCount)? as usize;
    let mut blend = [0u16; 8];
    for (i, b) in blend.iter_mut().enumerate() {
        *b = w.s.u16_at(s + X::blendVertCounts + 2 * i)?;
    }
    let vc = vert_count as usize;
    let verts = w.blob(s + X::verts0, 32 * vc, 15)?.map(|pos| Span { pos, len: 32 * vc });
    let tris = w.blob(s + X::triIndices, 6 * tri_count as usize, 15)?.map(|pos| Span { pos, len: 6 * tri_count as usize });
    let rv = L::XRigidVertList::SIZE;
    let mut rigid = Vec::new();
    if let Some(at) = w.inline_array(s + X::rigidVertLists, rv * rigid_n, 7)? {
        for i in 0..rigid_n {
            let r = at + rv * i;
            rigid.push((w.s.u16_at(r)?, w.s.u16_at(r + 2)?, w.s.u16_at(r + 4)?, w.s.u16_at(r + 6)?));
            use L::XSurfaceCollisionTree as T;
            if let Some(tree) = w.inline_array(r + L::XRigidVertList::collisionTree, T::SIZE, 7)? {
                let nodes = w.s.u32_at(tree + T::nodeCount)? as usize;
                w.blob(tree + T::nodes, 16 * nodes, 15)?;
                let leafs = w.s.u32_at(tree + T::leafCount)? as usize;
                w.blob(tree + T::leafs, 2 * leafs, 1)?;
            }
        }
    } else if let Some(v) = w.s.ptr_at(s + X::rigidVertLists)?.virtual_offset()
        && let Some(at) = w.vmap.file_pos(v)
    {
        for i in 0..rigid_n {
            let r = at + rv * i;
            rigid.push((w.s.u16_at(r)?, w.s.u16_at(r + 2)?, w.s.u16_at(r + 4)?, w.s.u16_at(r + 6)?));
        }
    }
    let blend_size = (w.s.u32_at(s + X::blendVertSize)? & !1) as usize;
    let blend_verts = w.blob(s + X::blendVerts, blend_size, 3)?.map(|pos| Span { pos, len: blend_size });
    w.blob(s + X::lmapUnwrap, 8 * vc, 3)?;
    use L::XSurfaceSubdivInfo as SI;
    use L::XSurfaceSubdivLevel as SL;
    if let Some(info) = w.inline_array(s + X::subdiv, SI::SIZE, 15)?
        && let Some(levels) = w.inline_array(info + SI::levels, SL::SIZE * subdiv_n, 7)?
    {
        for i in 0..subdiv_n {
            let l = levels + SL::SIZE * i;
            let u = |w: &Walker, o: usize| w.s.u32_at(l + o).map(|v| v as usize);
            let face_count = u(w, SL::faceCount)?;
            let patches = u(w, SL::regularPatchCount)?;
            let face_points = u(w, SL::facePointBufferSize)? & !3;
            let edge_points = u(w, SL::edgePointCount)?;
            let vertex_points = u(w, SL::vertexPointBufferSize)? & !3;
            let normals = u(w, SL::normalBufferSize)? & !3;
            let transitions = u(w, SL::transitionPointCount)?;
            w.blob(l + SL::rigidVertLists, 16 * rigid_n, 3)?;
            w.blob(l + SL::faceIndices, 12 * face_count, 3)?;
            w.blob(l + SL::regularPatchIndices, 32 * patches, 3)?;
            w.blob(l + SL::regularPatchFlags, 4 * patches, 3)?;
            w.blob(l + SL::facePoints, face_points, 3)?;
            w.blob(l + SL::edgePoints, 8 * edge_points, 3)?;
            w.blob(l + SL::vertexPoints, vertex_points, 3)?;
            w.blob(l + SL::normals, normals, 3)?;
            w.blob(l + SL::transitionPoints, 4 * transitions, 3)?;
            w.blob(l + SL::regularPatchCones, 32 * patches, 3)?;
        }
    }
    let blend_total: usize = blend.iter().map(|&b| b as usize).sum();
    w.blob(s + X::tensionData, 4 * blend_total, 3)?;
    w.blob(s + X::tensionAccumTable, 32 * vc, 1)?;
    let sh = if flags & 0x500 == 0x500 {
        32
    } else if flags & 0x100 != 0 {
        8
    } else if flags & 0x400 != 0 {
        24
    } else {
        0
    };
    if sh > 0 {
        w.blob(s + X::shProbeSimplexVertData, sh * vc, 3)?;
    }
    let mut part_bits = [0u32; 8];
    for (i, p) in part_bits.iter_mut().enumerate() {
        *p = w.s.u32_at(s + X::partBits + 4 * i)?;
    }
    Ok(SurfaceRec { flags, vert_count, tri_count, blend_vert_counts: blend, verts, tris, rigid, blend_verts, part_bits })
}

pub(super) fn xmodel_surfs(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::XModelSurfs as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    let n = w.s.u16_at(h + S::numsurfs)? as usize;
    let mut surfs = Vec::new();
    let xs = L::XSurface::SIZE;
    if let Some(at) = w.inline_array(h + S::surfs, xs * n, 3)? {
        for i in 0..n {
            surfs.push(surface(w, at + xs * i)?);
        }
    }
    w.s.pop();
    w.cap.model_surfs.push(ModelSurfsRec { name, surfs });
    Ok(AssetId { ty: AssetType::XModelSurfs, index: (w.cap.model_surfs.len() - 1) as u32 })
}
