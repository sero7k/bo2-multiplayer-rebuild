//! Sounds (not read yet).

use super::{AssetId, Walker};
use crate::cursor::{ReadError, Result};

/// A sound alias list (only its name for now).
#[derive(Clone, Debug, Default)]
pub struct SoundAliasList {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct LoadedSound {
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct SoundFile;

pub(super) fn sound(_w: &mut Walker, h: usize) -> Result<AssetId> {
    Err(ReadError::Invalid("sound not read yet", h))
}

pub(super) fn loaded_sound(_w: &mut Walker, h: usize) -> Result<AssetId> {
    Err(ReadError::Invalid("loaded sound not read yet", h))
}
