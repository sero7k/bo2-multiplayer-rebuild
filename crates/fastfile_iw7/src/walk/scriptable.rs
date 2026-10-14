//! Stub: filled in from x64-zt.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::{ReadError, Result};

pub(super) fn scriptable(_w: &mut Walker, h: usize) -> Result<AssetId> {
    Err(ReadError::Invalid(AssetType::Scriptable.name(), h))
}
