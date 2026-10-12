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
