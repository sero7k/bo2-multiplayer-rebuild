//! Readers for Call of Duty 4: Modern Warfare (IW3) zones: fast files, the
//! asset stream and its typed assets (world, collision, materials, images,
//! map entities), and `.iwi` textures.
//!
//! Ported from Nilbong/COD4-Rewrite's `iw3` crate (MIT OR Apache-2.0), whose
//! struct layouts follow the OpenAssetTools project's IW3 definitions.
//! Nothing here ships game data: everything is read from the player's own
//! Call of Duty 4 files.

pub mod ents;
pub mod fastfile;
pub mod iwi;
pub mod unpack;
pub mod wavelet;
pub mod zone;
