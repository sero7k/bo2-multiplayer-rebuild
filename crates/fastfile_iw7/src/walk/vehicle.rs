//! Stub: inferred from x64-zt's structs (no writer there).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};

pub(super) fn vehicle(_w: &mut Walker, h: usize) -> Result<AssetId> {
    Err(ReadError::Invalid(AssetType::Vehicle.name(), h))
}
