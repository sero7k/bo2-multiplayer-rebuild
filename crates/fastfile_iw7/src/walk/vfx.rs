//! Particle systems (x64-zt `iw7/assets/particle_system.cpp`): emitters,
//! their states, module groups and modules, in the writer's order. A
//! module's type selects which union member (and so which pointers) it has.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

use L::ParticleModuleDef as M;

/// `ParticleCurveDef`s at `field` (16 bytes each): their control points.
fn curves(w: &mut Walker, field: usize, n: usize) -> Result<()> {
    let point = L::ParticleCurveControlPointDef::SIZE;
    for i in 0..n {
        let c = field + L::ParticleCurveDef::SIZE * i;
        let k = w.s.u32_at(c + L::ParticleCurveDef::numControlPoints)? as usize;
        w.blob(c + L::ParticleCurveDef::controlPoints, point * k, 15)?;
    }
    Ok(())
}

/// One `ParticleLinkedAssetDef`: which member is set depends on the
/// owning module's type.
fn linked_asset(w: &mut Walker, ty: u32, a: usize) -> Result<()> {
    use AssetType as T;
    use L::ParticleLinkedAssetDef as D;
    match ty {
        0x0a => {
            w.asset(T::Material, a + D::material)?;
        }
        0x0c | 0x1a => {
            w.asset(T::XModel, a + D::model)?;
        }
        0x28 => {
            w.asset(T::PhysicsFxPipeline, a + D::physicsFXData__physicsFXPipeline)?;
            w.asset(T::PhysicsFxShape, a + D::physicsFXData__physicsFXShape)?;
        }
        0x0f => {
            w.asset(T::ParticleSimAnimation, a + D::particleSim)?;
        }
        0x05 => {
            for i in 0..3 {
                w.asset(T::Material, a + D::decal__materials + 8 * i)?;
            }
        }
        0x15 => {
            w.xstr(a + D::sound)?;
        }
        0x1d => {
            w.asset(T::VectorField, a + D::vectorField)?;
        }
        0x08 | 0x09 => {
            w.asset(T::LightDef, a + D::lightDef)?;
        }
        0x02 | 0x14 | 0x30..=0x35 => {
            w.asset(T::Vfx, a + D::particleSystem)?;
        }
        _ => {}
    }
    Ok(())
}

/// A `ParticleLinkedAssetListDef` at `field`.
fn linked_list(w: &mut Walker, ty: u32, field: usize) -> Result<()> {
    use L::ParticleLinkedAssetListDef as D;
    let n = w.s.u32_at(field + D::numAssets)? as usize;
    let len = L::ParticleLinkedAssetDef::SIZE;
    if let Some(list) = w.inline_array(field + D::assetList, len * n, 7)? {
        for i in 0..n {
            linked_asset(w, ty, list + len * i)?;
        }
    }
    Ok(())
}

fn module(w: &mut Walker, m: usize) -> Result<()> {
    let ty = w.s.u32_at(m + M::moduleType)?;
    let list = |w: &mut Walker, f: usize| linked_list(w, ty, m + f);
    match ty {
        0x02 => list(w, M::moduleData__initBeam__m_linkedAssetList)?,
        0x04 => curves(w, m + M::moduleData__initCloud__curves, 2)?,
        0x05 => list(w, M::moduleData__initDecal__m_linkedAssetList)?,
        0x06 => curves(w, m + M::moduleData__initFlare__m_curves, 4)?,
        0x08 => list(w, M::moduleData__initLightOmni__m_linkedAssetList)?,
        0x09 => list(w, M::moduleData__initLightSpot__m_linkedAssetList)?,
        0x0a => list(w, M::moduleData__initMaterial__m_linkedAssetList)?,
        0x0c => list(w, M::moduleData__initModel__m_linkedAssetList)?,
        0x0f => list(w, M::moduleData__initParticleSim__m_linkedAssetList)?,
        0x14 => list(w, M::moduleData__initRunner__m_linkedAssetList)?,
        0x15 => list(w, M::moduleData__initSound__m_linkedAssetList)?,
        0x16 => curves(w, m + M::moduleData__initSpawn__m_curves, 1)?,
        0x1a => {
            list(w, M::moduleData__initSpawnShapeMesh__m_linkedAssetList)?;
            let n = w.s.u32_at(m + M::moduleData__initSpawnShapeMesh__m_numMeshAssets)? as usize;
            w.blob(
                m + M::moduleData__initSpawnShapeMesh__m_meshAssetData,
                L::ParticleSpawnMeshAssetDef::SIZE * n,
                3,
            )?;
        }
        0x1d => list(w, M::moduleData__initVectorField__m_linkedAssetList)?,
        0x20 => curves(w, m + M::moduleData__colorGraph__m_curves, 8)?,
        0x22 => curves(w, m + M::moduleData__emissionGraph__m_curves, 2)?,
        0x23 => curves(w, m + M::moduleData__emissiveGraph__m_curves, 2)?,
        0x26 => curves(w, m + M::moduleData__intensityGraph__m_curves, 2)?,
        0x27 => curves(w, m + M::moduleData__parentVelocityGraph__m_curves, 2)?,
        0x28 => list(w, M::moduleData__physicsLight__m_linkedAssetList)?,
        0x2a => curves(w, m + M::moduleData__positionGraph__m_curves, 6)?,
        0x2b => curves(w, m + M::moduleData__rotationGraph__m_curves, 2)?,
        0x2c => curves(w, m + M::moduleData__rotationGraph3D__m_curves, 6)?,
        0x2d => curves(w, m + M::moduleData__sizeGraph__m_curves, 6)?,
        0x2f => curves(w, m + M::moduleData__velocityGraph__m_curves, 6)?,
        0x30 => list(w, M::moduleData__testAge__m_eventHandlerData__m_linkedAssetList)?,
        0x31 => list(w, M::moduleData__testBirth__m_eventHandlerData__m_linkedAssetList)?,
        0x32 => list(w, M::moduleData__testDeath__m_eventHandlerData__m_linkedAssetList)?,
        0x33 => list(w, M::moduleData__testGravity__m_eventHandlerData__m_linkedAssetList)?,
        0x34 => list(w, M::moduleData__testImpact__m_eventHandlerData__m_linkedAssetList)?,
        0x35 => list(w, M::moduleData__testPos__m_eventHandlerData__m_linkedAssetList)?,
        _ => {}
    }
    Ok(())
}

fn module_group(w: &mut Walker, g: usize) -> Result<()> {
    use L::ParticleModuleGroupDef as G;
    let n = w.s.u32_at(g + G::numModules)? as usize;
    if let Some(mods) = w.inline_array(g + G::moduleDefs, M::SIZE * n, 15)? {
        for i in 0..n {
            module(w, mods + M::SIZE * i)?;
        }
    }
    Ok(())
}

fn state(w: &mut Walker, s: usize) -> Result<()> {
    let len = L::ParticleModuleGroupDef::SIZE;
    if let Some(groups) = w.inline_array(s + L::ParticleStateDef::moduleGroupDefs, len * 3, 15)? {
        for i in 0..3 {
            module_group(w, groups + len * i)?;
        }
    }
    Ok(())
}

fn emitter(w: &mut Walker, e: usize) -> Result<()> {
    use L::ParticleEmitterDef as E;
    let n = w.s.u32_at(e + E::numStates)? as usize;
    let len = L::ParticleStateDef::SIZE;
    if let Some(states) = w.inline_array(e + E::stateDefs, len * n, 15)? {
        for i in 0..n {
            state(w, states + len * i)?;
        }
    }
    curves(w, e + E::fadeCurveDef, 1)
}

pub(super) fn vfx(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ParticleSystemDef as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + P::numEmitters)? as usize;
    let len = L::ParticleEmitterDef::SIZE;
    if let Some(emitters) = w.inline_array(h + P::emitterDefs, len * n, 15)? {
        for i in 0..n {
            emitter(w, emitters + len * i)?;
        }
    }
    let n = w.s.u32_at(h + P::numScriptedInputNodes)? as usize;
    w.blob(h + P::scriptedInputNodeDefs, L::ParticleScriptedInputNodeDef::SIZE * n, 15)?;
    w.s.pop();
    Ok(w.name_only(AssetType::Vfx, name))
}

/// A particle simulation (x64-zt `iw7/assets/fxparticlesimanimation.cpp`):
/// its material, particle data (aligned 32), 3-D orientations, indices,
/// frames and colour table.
pub(super) fn particle_sim(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxParticleSimAnimation as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    w.asset(AssetType::Material, h + P::material)?;
    let n = w.s.u32_at(h + P::header__particleDataCount)? as usize;
    w.blob(h + P::particleData, L::FxParticleSimAnimationParticleData::SIZE * n, 31)?;
    w.blob(h + P::particleDataOrientation3D, L::FxParticleSimAnimationParticleDataOrientation3D::SIZE * n, 7)?;
    w.blob(h + P::particleDataIndex, L::FxParticleSimAnimationParticleDataIndex::SIZE * n, 1)?;
    let frames = w.s.u32_at(h + P::header__frameCount)? as usize;
    w.blob(h + P::frames, L::FxParticleSimAnimationFrame::SIZE * frames, 3)?;
    let colours = w.s.u32_at(h + P::header__colorTableSize)? as usize;
    w.blob(h + P::colorTable, 16 * colours, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::ParticleSimAnimation, name))
}
