//! Map assets.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;
use crate::walk::{ComWorldRec, Span};

pub(super) fn com_world(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::ComWorld as C;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + C::name)?.unwrap_or_default();
    let n = w.s.u32_at(h + C::primaryLightCount)? as usize;
    let pl = L::ComPrimaryLight::SIZE;
    let mut primary_lights = None;
    if let Some(at) = w.inline_array(h + C::primaryLights, pl * n, 3)? {
        primary_lights = Some(Span {
            pos: at,
            len: pl * n,
        });
        for i in 0..n {
            w.xstr(at + pl * i + L::ComPrimaryLight::defName)?;
        }
    }
    let e = w.s.u32_at(h + C::primaryLightEnvCount)? as usize;
    w.blob(h + C::primaryLightEnvs, L::ComPrimaryLightEnv::SIZE * e, 3)?;
    w.s.pop();
    w.cap.com_worlds.push(ComWorldRec {
        name,
        record: Span {
            pos: h,
            len: C::SIZE,
        },
        primary_lights,
        primary_light_count: n as u32,
    });
    Ok(AssetId {
        ty: AssetType::ComWorld,
        index: (w.cap.com_worlds.len() - 1) as u32,
    })
}

/// `FxWorld`: its name and breakable-glass system. The glass definitions
/// (materials, physics, effects, sounds) are in the file; the pieces'
/// working arrays are RUNTIME allocations (no file data); the initial
/// piece states and geometry follow in VIRTUAL (x64-zt's `fx_world::write`
/// order).
///
/// PROVISIONAL: on retail mp_prisonbreak this reads 0x40 bytes short of
/// the next asset. The first glass def's three sound slots read as
/// all-ones values (not the `fd`-filled "follow"), and the later defs
/// reference them 0x5d8 bytes further on, so the sound names' real
/// encoding is not settled yet.
pub(super) fn fx_world(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::FxGlassSystem as G;
    use L::FxWorld as F;
    use crate::stream::RUNTIME;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + F::name)?.unwrap_or_default();
    let g = h + F::glassSys;
    let u = |w: &Walker, off: usize| -> Result<usize> { Ok(w.s.u32_at(g + off)? as usize) };
    let defs = u(w, G::defCount)?;
    let piece_limit = u(w, G::pieceLimit)?;
    let words = u(w, G::pieceWordCount)?;
    let init_pieces = u(w, G::initPieceCount)?;
    let cells = u(w, G::cellCount)?;
    let geo_limit = u(w, G::geoDataLimit)?;
    let init_geo = u(w, G::initGeoDataCount)?;
    let d = L::FxGlassDef::SIZE;
    if let Some(at) = w.inline_array(g + G::defs, d * defs, 3)? {
        for i in 0..defs {
            let def = at + d * i;
            use L::FxGlassDef as D;
            w.asset(AssetType::Material, def + D::material)?;
            w.asset(AssetType::Material, def + D::materialShattered)?;
            w.asset(AssetType::PhysPreset, def + D::physPreset)?;
            for fx in [D::pieceBreakEffect, D::shatterEffect, D::shatterSmallEffect, D::crackDecalEffect] {
                w.asset(AssetType::Fx, def + fx)?;
            }
            for snd in [D::damagedSound, D::destroyedSound, D::destroyedQuietSound] {
                super::misc::sound_custom(w, def + snd)?;
            }
        }
    }
    w.s.push(RUNTIME);
    w.blob(g + G::piecePlaces, L::FxGlassPiecePlace::SIZE * piece_limit, 3)?;
    w.blob(g + G::pieceStates, L::FxGlassPieceState::SIZE * piece_limit, 3)?;
    w.blob(g + G::pieceDynamics, L::FxGlassPieceDynamics::SIZE * piece_limit, 7)?;
    w.blob(g + G::geoData, L::FxGlassGeometryData::SIZE * geo_limit, 3)?;
    w.blob(g + G::isInUse, 4 * words, 3)?;
    w.blob(g + G::cellBits, 4 * words * cells, 3)?;
    w.blob(g + G::visData, (piece_limit + 15) & !15, 15)?;
    w.blob(g + G::linkOrg, 12 * piece_limit, 3)?;
    w.blob(g + G::halfThickness, 4 * ((piece_limit + 3) & !3), 15)?;
    w.s.pop();
    w.blob(g + G::lightingHandles, 2 * init_pieces, 1)?;
    w.blob(g + G::initPieceStates, L::FxGlassInitPieceState::SIZE * init_pieces, 3)?;
    w.blob(g + G::initGeoData, L::FxGlassGeometryData::SIZE * init_geo, 3)?;
    w.s.pop();
    Ok(w.name_only(AssetType::FxWorld, name))
}
