//! Readers for Call of Duty: World at War (T4) zones, so its multiplayer
//! maps can be played under Black Ops II's rules alongside CoD4's.
//!
//! Ported from Nilbong/COD4-Rewrite's `t4` crate (MIT OR Apache-2.0), whose
//! schema is generated from the OpenAssetTools project's T4 definitions
//! (`T4_Assets.h`, `XAssets/*.txt`). Nothing here ships game data:
//! everything is read from the player's own World at War files.
//!
//! * [`fastfile`] – `.ff` zone containers (CoD4's container, version 0x183)
//! * [`zone`] – the asset stream parser (schema-driven)
//! * [`convert`] – a map's world, collision, entities, models, materials and
//!   images as `iw3` assets ([`convert::to_iw3_map`])
//!
//! Images are CoD4's `.iwi` (version 6) files in the `.iwd` archives.

pub mod convert;
pub mod fastfile;
pub mod zone;

/// Read a World at War map zone (`mp_airfield.ff`, ...) and convert it to
/// an `iw3` zone. Parsing stops at the first asset it cannot read; what was
/// read before is kept (`zone.stats.stopped_at` says where it stopped).
pub fn load_map(path: &std::path::Path) -> anyhow::Result<convert::MapZone> {
    let data = fastfile::load(path)?;
    let parsed = zone::Zone::parse(&data, zone::ParseOptions::default())?;
    Ok(convert::to_iw3_map(&parsed))
}

#[cfg(test)]
mod tests;
