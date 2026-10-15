//! Call of Duty: Modern Warfare III (2023, internal "jup", `cod23`), PC.
//! The container layer only: getting zone bytes and package entries out of
//! the owner's install.
//!
//! - [`container`]: the `.ff` envelope (`IWffa100`), its Oodle blocks and
//!   master blocks, down to the decompressed zone image.
//! - [`xsub`]: the `KAPI` packages (`.xsub`, `.xpak`) of streamed data:
//!   header, hash table, and one entry fetched and decompressed.
//! - [`oodle`]: the [`Decompressor`] trait both go through. The game's own
//!   `oo2core_8_win64.dll` is loaded at runtime on Windows (`oodle_dll`, the
//!   only module with `unsafe`); nothing of Oodle is shipped.
//!
//! What is inside a zone (asset list, layouts) is not this crate's business
//! yet. See `docs/JUP.md`.
#![deny(unsafe_code)]

pub mod container;
pub mod map;
pub mod oodle;
#[cfg(windows)]
pub mod oodle_dll;
pub mod xsub;

pub use container::{
    BlockRef, ContainerError, FileHeader, inflate_zone, is_jup_zone, parse_file_header, read_zone,
    walk_blocks, walk_blocks_in,
};
pub use oodle::{Decompressor, OodleError, StoredOnly};
pub use xsub::{XPak, XSub, XSubEntry, XSubError, XSubHeader};
