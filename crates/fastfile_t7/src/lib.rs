//! Call of Duty: Black Ops III (T7), PC x64 zones.
//!
//! The `.ff` is a fixed 0x248-byte `XFile` header (`container`) followed by
//! zlib blocks that inflate to the zone image. The image starts with the
//! `XAssetList` (script strings, depends, the `XAsset` table; `xfile`), then
//! every asset's data in table order, packed with no padding. Pointers are
//! 8 bytes: `-1` data follows, `-2` data follows ("insert"), `block << 60 |
//! offset + 1` data loaded earlier (`ptr`). Asset types and the x64 header
//! sizes are in `asset_type`; `walk` reads the assets whose layout is
//! known, in table order, the way the loader does (`stream`: block by
//! block, with the loader's alignment), checks every reference against what
//! it has read, and reports where it has to stop.
//!
//! Streamed images and meshes keep their data in `.xpak` files, looked up
//! by 64-bit key (`xpak`: hash table, LZ4 chunks); the zone has the
//! entries' key/offset/size (`walk::image`, `walk::model`); small images
//! have their pixels in the zone.
#![forbid(unsafe_code)]

pub mod asset_type;
pub mod container;
pub mod cursor;
pub mod map;
pub mod mesh;
pub mod ptr;
pub mod sab;
pub mod stream;
pub mod texture;
pub mod ui_image;
pub mod walk;
pub mod xfile;
pub mod xpak;
pub mod zone;

pub use asset_type::AssetType;
pub use container::{ContainerError, FileHeader, ZONE_VERSION, inflate_zone, is_t7_zone, parse_file_header};
pub use ptr::Ptr;
pub use xfile::{AssetEntry, AssetList, parse_asset_list};
pub use zone::{Zone, ZoneError};
