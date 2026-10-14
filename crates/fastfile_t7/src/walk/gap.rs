//! Gaps: inline assets of types whose layout is not read yet but whose
//! extent the walk can still find, so that it goes on past them.
//!
//! Only `customizationtable` so far: `core_common`'s character tables
//! (`cp_character_customization`, `cp_cpzm_character_customization`, ...;
//! body / helmet / head types with their icons) sit between the campaign
//! weapons and the multiplayer and zombies ones, 1.6 MiB of nested data.
//!
//! The walk finds where the next asset starts (`resume_point`: its header's
//! name pointer `-1`, a plausible name right after the header, and the
//! next few assets reading from there on a scratch stream, every one of
//! them, so that a header-like run inside the gap is not taken), steps
//! there, and counts the gap's bytes as VIRTUAL. Part of a gap is in other
//! blocks (image pixels in PHYSICAL), so that count is too high; the
//! references read after the gap then miss their targets by the same
//! amount, and `fit` finds it: the shift that lands most of them on starts
//! read after the gap. `walk_with` walks again with the fitted size, gap
//! by gap, until every gap has one.

use super::{is_walkable, trial_strict};
use crate::asset_type::AssetType;
use crate::zone::Zone;

/// Types the walk steps over as gaps.
pub fn is_gap(ty: AssetType) -> bool {
    matches!(ty, AssetType::CustomizationTable)
}

/// A gap the walk stepped over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    /// The first asset of the gap and the asset the walk resumed at.
    pub index: usize,
    pub resumed_at: usize,
    /// File range stepped over: the first asset's header to the resumed one's.
    pub start: usize,
    pub end: usize,
    /// VIRTUAL position where the gap starts and the size counted for it.
    pub virtual_start: u64,
    pub virtual_len: u64,
    /// The fitted size: (references that landed with it, of those tried);
    /// None while the walk used the byte count.
    pub fitted: Option<(usize, usize)>,
}

/// The next asset after the gap that starts at asset `index`: the first
/// inline asset after it of another type, and where its header is, at or
/// after file offset `from` (within 64 MiB).
pub fn resume_point(zone: &Zone, index: usize, from: usize) -> Option<(usize, usize)> {
    const LIMIT: usize = 64 << 20;
    let assets = &zone.assets.assets;
    let failed = assets.get(index)?.asset_type()?;
    let (j, ty) = assets
        .iter()
        .enumerate()
        .skip(index + 1)
        .filter(|(_, a)| a.is_inline())
        .find_map(|(j, a)| a.asset_type().filter(|t| *t != failed).map(|t| (j, t)))?;
    if !is_walkable(ty) || ty.header_size() == 0 {
        return None;
    }
    let data = &zone.image;
    let (size, nf) = (ty.header_size(), super::name_field(ty));
    let end = data.len().min(from.saturating_add(LIMIT));
    let mut p = from;
    while p + size + 8 < end {
        if data[p + nf..p + nf + 8] == [0xff; 8] && super::plausible_name(data, p + size) && trial_strict(zone, j, p) {
            return Some((j, p));
        }
        p += 1;
    }
    None
}

/// The shift (our VIRTUAL count minus the zone's) that lands most of
/// `misses` (field, target) on `starts` (sorted, the starts read after the
/// gap), looked for up to `range` away: (shift, landed, tried).
pub fn fit(misses: &[(usize, u64)], starts: &[u64], range: u64) -> Option<(i64, usize, usize)> {
    use std::collections::HashMap;
    let sample: Vec<u64> = misses.iter().map(|m| m.1).take(64).collect();
    if sample.is_empty() {
        return None;
    }
    let mut votes: HashMap<i64, usize> = HashMap::new();
    for &t in &sample {
        let lo = starts.partition_point(|&s| s < t.saturating_sub(range));
        let hi = starts.partition_point(|&s| s <= t.saturating_add(range));
        for &s in &starts[lo..hi] {
            *votes.entry(s as i64 - t as i64).or_default() += 1;
        }
    }
    let (d, n) = votes.into_iter().max_by_key(|&(d, n)| (n, -d.abs()))?;
    (n * 2 > sample.len()).then_some((d, n, sample.len()))
}
