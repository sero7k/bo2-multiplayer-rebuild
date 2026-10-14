//! Spaceship targets: no x64-zt writer for IW7. `SpaceshipTargetDef`
//! (x64-zt `iw7/structs.hpp`) has a single pointer, its name; the rest is
//! a script string and floats.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

pub(super) fn spaceship(w: &mut Walker, h: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = w.xstr(h + L::SpaceshipTargetDef::name)?.unwrap_or_default();
    w.s.pop();
    Ok(w.name_only(AssetType::SpaceshipTarget, name))
}
