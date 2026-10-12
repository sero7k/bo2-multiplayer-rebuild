//! Effects (`FxEffectDef`) and particle simulation animations.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{FxElemRec, FxRec, Span};

const FX_MODEL: u8 = 11;
const FX_OMNI_LIGHT: u8 = 12;
const FX_SPOT_LIGHT: u8 = 13;
const FX_SOUND: u8 = 14;
const FX_DECAL: u8 = 15;
const FX_RUNNER: u8 = 16;
const FX_VECTORFIELD: u8 = 17;
const FX_PARTICLE_SIM: u8 = 5;
const FX_TRAIL: u8 = 3;
const FX_FLARE: u8 = 4;
const FX_SPARK_FOUNTAIN: u8 = 10;

/// One visual (`FxElemVisuals`, a pointer-sized union) at `field`.
fn visual(w: &mut Walker, elem_type: u8, field: usize, rec: &mut FxElemRec) -> Result<()> {
    if w.s.u64_at(field)? == 0 {
        return Ok(());
    }
    match elem_type {
        FX_MODEL => {
            if let Some(id) = w.asset(AssetType::XModel, field)? {
                rec.models.push(id);
            }
        }
        FX_RUNNER => {
            if let Some(name) = w.xstr(field)? {
                rec.effects.push(name);
            }
        }
        FX_SOUND => {
            if let Some(name) = w.xstr(field)? {
                rec.sounds.push(name);
            }
        }
        FX_VECTORFIELD => {
            w.xstr(field)?;
        }
        FX_PARTICLE_SIM => {
            w.asset(AssetType::ParticleSimAnimation, field)?;
        }
        FX_OMNI_LIGHT => {}
        FX_SPOT_LIGHT => {
            w.asset(AssetType::LightDef, field)?;
        }
        _ => {
            if let Some(id) = w.asset(AssetType::Material, field)? {
                rec.materials.push(id);
            }
        }
    }
    Ok(())
}

fn elem(w: &mut Walker, e: usize) -> Result<FxElemRec> {
    use L::FxElemDef as E;
    let elem_type = w.s.u8_at(e + E::elemType)?;
    let mut rec = FxElemRec { elem_type, record: Span { pos: e, len: E::SIZE }, ..Default::default() };
    let vel = w.s.u8_at(e + E::velIntervalCount)? as usize + 1;
    w.blob(e + E::velSamples, L::FxElemVelStateSample::SIZE * vel, 3)?;
    let vis = w.s.u8_at(e + E::visStateIntervalCount)? as usize + 1;
    w.blob(e + E::visSamples, L::FxElemVisStateSample::SIZE * vis, 3)?;
    let count = w.s.u8_at(e + E::visualCount)? as usize;
    if elem_type == FX_DECAL {
        let mv = L::FxElemMarkVisuals::SIZE;
        if let Some(marks) = w.inline_array(e + E::visuals, mv * count, 3)? {
            for i in 0..count {
                for k in 0..3 {
                    if let Some(id) = w.asset(AssetType::Material, marks + mv * i + 8 * k)? {
                        rec.materials.push(id);
                    }
                }
            }
        }
    } else if count > 1 {
        if let Some(array) = w.inline_array(e + E::visuals, 8 * count, 3)? {
            for i in 0..count {
                visual(w, elem_type, array + 8 * i, &mut rec)?;
            }
        }
    } else {
        visual(w, elem_type, e + E::visuals, &mut rec)?;
    }
    for f in [E::effectOnImpact, E::effectOnDeath, E::effectEmitted] {
        if let Some(name) = w.xstr(e + f)? {
            rec.effects.push(name);
        }
    }
    let ext = e + E::extended;
    if w.s.u64_at(ext)? != 0 {
        match elem_type {
            FX_TRAIL => {
                use L::FxTrailDef as T;
                if let Some(t) = w.inline_array(ext, T::SIZE, 3)? {
                    let verts = w.s.u32_at(t + T::vertCount)? as usize;
                    w.blob(t + T::verts, L::FxTrailVertex::SIZE * verts, 3)?;
                    let inds = w.s.u32_at(t + T::indCount)? as usize;
                    w.blob(t + T::inds, 2 * inds, 1)?;
                }
            }
            FX_SPARK_FOUNTAIN => {
                w.blob(ext, L::FxSparkFountainDef::SIZE, 3)?;
            }
            FX_SPOT_LIGHT => {
                w.blob(ext, L::FxSpotLightDef::SIZE, 3)?;
            }
            FX_FLARE => {
                use L::FxFlareDef as F;
                if let Some(f) = w.inline_array(ext, F::SIZE, 3)? {
                    for (data, count) in [
                        (F::intensityX, F::intensityXIntervalCount),
                        (F::intensityY, F::intensityYIntervalCount),
                        (F::srcCosIntensity, F::srcCosIntensityIntervalCount),
                        (F::srcCosScale, F::srcCosScaleIntervalCount),
                    ] {
                        let n = w.s.u32_at(f + count)? as usize + 1;
                        w.blob(f + data, 4 * n, 3)?;
                    }
                }
            }
            _ => {
                w.blob(ext, 1, 0)?;
            }
        }
    }
    Ok(rec)
}

pub(super) fn effect(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxEffectDef as F;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + F::name)?.unwrap_or_default();
    let counts = (w.s.i32_at(h + F::elemDefCountLooping)?, w.s.i32_at(h + F::elemDefCountOneShot)?, w.s.i32_at(h + F::elemDefCountEmission)?);
    let n = (counts.0 + counts.1 + counts.2).max(0) as usize;
    let ed = L::FxElemDef::SIZE;
    let mut elems = Vec::new();
    if let Some(defs) = w.inline_array(h + F::elemDefs, ed * n, 3)? {
        for i in 0..n {
            elems.push(elem(w, defs + ed * i)?);
        }
    }
    w.s.pop();
    w.cap.fx.push(FxRec { name, flags: w.s.i32_at(h + F::flags)?, elem_counts: counts, elems, record: Span { pos: h, len: F::SIZE } });
    Ok(AssetId { ty: AssetType::Fx, index: (w.cap.fx.len() - 1) as u32 })
}

pub(super) fn particle_sim_animation(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxParticleSimAnimation as P;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + P::name)?.unwrap_or_default();
    w.asset(AssetType::Material, h + P::material)?;
    let particles = w.s.u32_at(h + P::header__particleDataCount)? as usize;
    w.blob(h + P::particleData, L::FxParticleSimAnimationParticleData::SIZE * particles, 31)?;
    let frames = w.s.u32_at(h + P::header__frameCount)? as usize;
    w.blob(h + P::frames, L::FxParticleSimAnimationFrame::SIZE * frames, 3)?;
    let colors = w.s.u32_at(h + P::header__colorTableSize)? as usize;
    w.blob(h + P::colorTable, 16 * colors, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::ParticleSimAnimation, name))
}
