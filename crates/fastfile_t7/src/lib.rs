//! Call of Duty: Black Ops III (T7), PC x64 zones.
//!
//! The `.ff` is a fixed 0x248-byte `XFile` header (`container`) followed by
//! zlib blocks that inflate to the zone image. The image starts with the
//! `XAssetList` (script strings, depends, the `XAsset` table; `xfile`), then
//! every asset's data in table order, packed with no padding. Pointers are
//! 8 bytes: `-1` data follows, `-2` data follows ("insert"), `block << 60 |
//! offset + 1` data loaded earlier (`ptr`). Asset types and the x64 header
//! sizes are in `asset_type`; `walk` steps over the assets whose layout is
//! known, in table order, and reports where it has to stop.
//!
//! Images are not in the zones: their pixels live in `.xpak` files (LZ4 per
//! block), which this crate does not read yet.
#![forbid(unsafe_code)]

pub mod asset_type;
pub mod container;
pub mod cursor;
pub mod ptr;
pub mod walk;
pub mod xfile;
pub mod zone;

pub use asset_type::AssetType;
pub use container::{ContainerError, FileHeader, ZONE_VERSION, inflate_zone, is_t7_zone, parse_file_header};
pub use ptr::Ptr;
pub use xfile::{AssetEntry, AssetList, parse_asset_list};
pub use zone::{Zone, ZoneError};
