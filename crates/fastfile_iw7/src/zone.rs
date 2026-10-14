//! An opened Infinite Warfare zone: the envelope, the inflated image and its asset table.

use std::path::Path;

use crate::container::{ContainerError, FileHeader, inflate_zone};
use crate::cursor::ReadError;
use crate::xfile::{AssetList, parse_asset_list};

#[derive(Debug)]
pub enum ZoneError {
    Container(ContainerError),
    Read(ReadError),
}

impl std::fmt::Display for ZoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Container(e) => e.fmt(f),
            Self::Read(e) => write!(f, "zone image: {e}"),
        }
    }
}

impl std::error::Error for ZoneError {}

impl From<ContainerError> for ZoneError {
    fn from(e: ContainerError) -> Self {
        Self::Container(e)
    }
}
impl From<ReadError> for ZoneError {
    fn from(e: ReadError) -> Self {
        Self::Read(e)
    }
}

pub struct Zone {
    pub header: FileHeader,
    /// The inflated zone (the stream from its asset list on).
    pub image: Vec<u8>,
    pub assets: AssetList,
}

impl Zone {
    pub fn from_file_bytes(file: &[u8]) -> Result<Self, ZoneError> {
        let (header, image) = inflate_zone(file)?;
        let assets = parse_asset_list(&image)?;
        Ok(Self {
            header,
            image,
            assets,
        })
    }

    pub fn open(path: &Path) -> Result<Self, ZoneError> {
        let file = std::fs::read(path).map_err(ContainerError::Io)?;
        Self::from_file_bytes(&file)
    }

}
