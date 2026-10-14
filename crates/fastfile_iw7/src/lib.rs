//! Call of Duty: Infinite Warfare (IW7), PC x64 zones.
//!
//! IW7's zones are Advanced Warfare's lineage: an `IWC` LZ4 envelope
//! (`container`), the block sizes in the file header, ten blocks (VIRTUAL
//! is 8), a `0xFFFFFFF` pointer fill (`ptr`), and IW7's own asset types
//! and struct layouts (`asset_type`, `layout`, from x64-zt's IW7 headers).
//! The guns: weapon definitions in the common zones, each gun's models in
//! its own transient zones (`mp_weapon_view_iw7_<gun>_tr` first person,
//! `mp_weapon_world_iw7_<gun>_tr` world).
#![forbid(unsafe_code)]

pub mod asset_type;
pub mod container;
pub mod cursor;
pub mod layout;
pub mod map;
pub mod pak;
pub mod ptr;
pub mod stream;
pub mod texture;
pub mod ui_image;
pub mod walk;
pub mod xfile;
pub mod zone;

pub use asset_type::AssetType;
pub use container::{ContainerError, FileHeader, ZONE_VERSION, inflate_zone, is_iw7_zone, parse_file_header};
pub use ptr::Ptr;
pub use xfile::{AssetEntry, AssetList, parse_asset_list};
pub use zone::{Zone, ZoneError};
