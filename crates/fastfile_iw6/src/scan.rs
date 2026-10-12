//! Finding a zone's sound alias lists without reading every asset before them.
//!
//! A full walk of an IW6 zone needs a reader for every asset type that comes
//! first (materials, techsets, models ...). Until those exist, the sound
//! lists are found by their shape instead: a `snd_alias_list_t` header
//! (`name` inline or a VIRTUAL reference, `head` "follow", `count` 1..255)
//! followed by an alias array whose every entry names itself like the list.
//! Each candidate is read strictly (`sound.rs`); a list that reads ends
//! exactly where the next one begins when they are adjacent in the asset
//! table, which the scan counts (`chained`) as its own check.
//!
//! Names and shared sound files are references to VIRTUAL offsets. Their
//! targets are found three ways:
//! * the `NetConstStrings` at the head of the zone are read exactly from
//!   the table's end, so their strings' VIRTUAL offsets are known;
//! * a list with its own (inline) name is an anchor: its aliases point at
//!   that name, which gives the VIRTUAL offset there, and the count carries
//!   through the following adjacent lists (`anchored`; an anchor that
//!   disagrees with the carried count is a `conflict`, and there are none
//!   in the retail zones read so far);
//! * a run of adjacent lists with no anchor is placed where the shared
//!   sound-file references of the zone land on its inline sound files
//!   (`calibrated`, accepted with at least two agreeing references).

use std::collections::{HashMap, HashSet};

use crate::asset_type::AssetType;
use crate::ncs::{self, NetConstStrings, StringPool};
use crate::ptr::FOLLOW_BYTES;
use crate::sound::{
    LoadedRef, Name, SND_ALIAS_LIST_LEN, SoundAliasList, SoundData, SoundFile, SoundFileRef, Trace,
    read_sound_alias_list,
};
use crate::xfile::AssetList;

#[derive(Clone, Debug, Default)]
pub struct SoundScan {
    pub lists: Vec<SoundAliasList>,
    /// Inline sound assets in the table (what the scan should find).
    pub expected: usize,
    pub chained: usize,
    pub anchored: usize,
    pub calibrated: usize,
    pub conflicts: usize,
    pub net_const_strings: Vec<NetConstStrings>,
}

impl SoundScan {
    pub fn find(&self, alias: &str) -> Option<&SoundAliasList> {
        self.lists
            .iter()
            .find(|l| l.name.text().is_some_and(|n| n.eq_ignore_ascii_case(alias)))
    }

    pub fn unresolved_names(&self) -> usize {
        self.lists
            .iter()
            .filter(|l| l.name.text().is_none())
            .count()
    }

    pub fn unresolved_sound_files(&self) -> usize {
        self.lists
            .iter()
            .flat_map(|l| &l.aliases)
            .filter(|a| matches!(a.sound, SoundFileRef::Shared { resolved: None, .. }))
            .count()
    }

    /// The streamed (pak) sounds, by alias.
    pub fn streamed(
        &self,
    ) -> impl Iterator<Item = (&SoundAliasList, &crate::sound::StreamedSound)> {
        self.lists.iter().flat_map(|l| {
            l.aliases
                .iter()
                .filter_map(move |a| a.sound.get().and_then(|f| f.data.stream()).map(|s| (l, s)))
        })
    }
}

fn find_follow(data: &[u8], from: usize) -> Option<usize> {
    let tail = data.get(from..)?;
    let mut i = 0;
    while i + 8 <= tail.len() {
        let p = tail[i..].iter().position(|&b| b == 0xFF)?;
        let at = i + p;
        if at + 8 > tail.len() {
            return None;
        }
        if tail[at..at + 8] == FOLLOW_BYTES {
            return Some(from + at);
        }
        i = at + 1;
    }
    None
}

/// Every well-formed sound alias list at or after `from`, in file order.
fn candidates(zone: &[u8], from: usize, expected: usize) -> Vec<SoundAliasList> {
    let mut out = Vec::new();
    let mut search = from + 8;
    while out.len() < expected
        && let Some(q) = find_follow(zone, search)
    {
        let o = q - 8;
        if o >= from
            && let Ok((list, _)) = read_sound_alias_list(zone, o, None)
        {
            search = list.end.max(q + 1);
            out.push(list);
            continue;
        }
        search = q + 1;
    }
    out
}

struct Learned {
    pool: StringPool,
    sound_files: HashMap<u64, (usize, usize)>,
}

impl Learned {
    fn take(&mut self, list_index: usize, trace: Trace) {
        for (v, s) in trace.strings {
            self.pool.entry(v).or_insert(s);
        }
        for (v, alias) in trace.sound_files {
            self.sound_files.entry(v).or_insert((list_index, alias));
        }
    }
}

/// The VIRTUAL offset of an inline-named list's name (its first alias's reference).
fn anchor_of(list: &SoundAliasList) -> Option<u64> {
    if !matches!(list.name, Name::Inline(_)) {
        return None;
    }
    match list.aliases.first()?.name {
        Name::Ref { virt, .. } => Some(virt),
        _ => None,
    }
}

fn reread(zone: &[u8], list: &SoundAliasList, virt: u64) -> Option<Trace> {
    read_sound_alias_list(zone, list.offset, Some(virt))
        .ok()
        .map(|(_, t)| t)
}

/// Places a run of adjacent lists with no anchor from the zone's shared
/// sound-file references; returns the VIRTUAL offset of its start.
fn calibrate(zone: &[u8], run: &[SoundAliasList], wanted: &HashSet<u64>) -> Option<u64> {
    if wanted.is_empty() {
        return None;
    }
    let mut best: Option<(u64, usize)> = None;
    for residue in 0..8u64 {
        let mut v = residue;
        let mut targets = Vec::new();
        for list in run {
            let trace = reread(zone, list, v)?;
            targets.extend(trace.sound_files.iter().map(|(t, _)| t - residue));
            v = trace.end_virt?;
        }
        let mut votes: HashMap<u64, usize> = HashMap::new();
        for &r in wanted {
            for &t in &targets {
                if let Some(base) = r.checked_sub(t)
                    && base % 8 == residue
                {
                    *votes.entry(base).or_default() += 1;
                }
            }
        }
        if let Some((&base, &n)) = votes
            .iter()
            .max_by_key(|(b, n)| (**n, std::cmp::Reverse(**b)))
            && best.is_none_or(|(_, m)| n > m)
        {
            best = Some((base, n));
        }
    }
    best.filter(|(_, n)| *n >= 2).map(|(b, _)| b)
}

pub fn scan_sounds(zone: &[u8], list: &AssetList) -> SoundScan {
    let mut learned = Learned {
        pool: StringPool::new(),
        sound_files: HashMap::new(),
    };
    let (net_const_strings, after) = ncs::read_head(zone, list, &mut learned.pool);
    let expected = list
        .assets
        .iter()
        .filter(|a| a.asset_type() == Some(AssetType::Sound) && a.is_inline())
        .count();
    let mut scan = SoundScan {
        expected,
        net_const_strings,
        ..SoundScan::default()
    };
    if expected == 0 {
        return scan;
    }
    let mut lists = candidates(zone, after.pos(), expected);

    // Runs of adjacent lists.
    let mut runs: Vec<std::ops::Range<usize>> = Vec::new();
    for i in 0..lists.len() {
        if i > 0 && lists[i - 1].end == lists[i].offset {
            scan.chained += 1;
            if let Some(r) = runs.last_mut() {
                r.end = i + 1;
            }
        } else {
            runs.push(i..i + 1);
        }
    }

    // Anchored walk; collect the stretches with no anchor.
    let mut unplaced: Vec<std::ops::Range<usize>> = Vec::new();
    for run in &runs {
        let mut v: Option<u64> = None;
        let mut first_anchor = None;
        for i in run.clone() {
            if let Some(anchor) = anchor_of(&lists[i]) {
                if v.is_some_and(|v| v != anchor) {
                    scan.conflicts += 1;
                }
                v = Some(anchor);
                first_anchor.get_or_insert(i);
            }
            let Some(at) = v else { continue };
            match reread(zone, &lists[i], at) {
                Some(trace) => {
                    v = trace.end_virt;
                    scan.anchored += 1;
                    learned.take(i, trace);
                }
                None => v = None,
            }
        }
        let end = first_anchor.unwrap_or(run.end);
        if end > run.start {
            unplaced.push(run.start..end);
        }
    }
    if !unplaced.is_empty() {
        let wanted: HashSet<u64> = lists
            .iter()
            .flat_map(|l| &l.aliases)
            .filter_map(|a| match a.sound {
                SoundFileRef::Shared { virt, .. } => Some(virt),
                _ => None,
            })
            .collect();
        for range in unplaced {
            let Some(mut v) = calibrate(zone, &lists[range.clone()], &wanted) else {
                continue;
            };
            for i in range {
                let Some(trace) = reread(zone, &lists[i], v) else {
                    break;
                };
                let Some(end) = trace.end_virt else { break };
                v = end;
                scan.calibrated += 1;
                learned.take(i, trace);
            }
        }
    }

    // Resolve.
    let inline_files: HashMap<u64, SoundFile> = learned
        .sound_files
        .iter()
        .filter_map(|(v, (li, ai))| match &lists[*li].aliases[*ai].sound {
            SoundFileRef::Inline(f) => Some((*v, f.clone())),
            _ => None,
        })
        .collect();
    for l in &mut lists {
        l.name.resolve(&learned.pool);
        let list_name = l.name.text().map(str::to_owned);
        for a in &mut l.aliases {
            for n in [
                &mut a.name,
                &mut a.subtitle,
                &mut a.secondary,
                &mut a.chain,
                &mut a.mixer_group,
            ] {
                n.resolve(&learned.pool);
            }
            if let (
                Name::Ref {
                    resolved: r @ None, ..
                },
                Some(ln),
            ) = (&mut a.name, &list_name)
            {
                *r = Some(ln.clone());
            }
            if let SoundFileRef::Shared { virt, resolved } = &mut a.sound {
                *resolved = inline_files.get(virt).cloned();
            }
            if let SoundFileRef::Inline(f) = &mut a.sound {
                resolve_file(f, &learned.pool);
            }
            if let SoundFileRef::Shared {
                resolved: Some(f), ..
            } = &mut a.sound
            {
                resolve_file(f, &learned.pool);
            }
        }
    }
    scan.lists = lists;
    scan
}

fn resolve_file(f: &mut SoundFile, pool: &StringPool) {
    let loaded = match &mut f.data {
        SoundData::Loaded(l) | SoundData::Primed { loaded_part: l, .. } => Some(l),
        _ => None,
    };
    if let Some(LoadedRef::Inline(l)) = loaded {
        l.name.resolve(pool);
    }
    if let SoundData::Streamed(s) | SoundData::Primed { stream: s, .. } = &mut f.data {
        for n in [&mut s.raw_dir, &mut s.raw_name].into_iter().flatten() {
            n.resolve(pool);
        }
    }
}

/// Bytes a list header takes before its name (for tools that print offsets).
pub const LIST_HEADER_LEN: usize = SND_ALIAS_LIST_LEN;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ncs::tests::ncs_bytes;
    use crate::ptr::{FOLLOW, Ptr};
    use crate::sound::tests::{alias, loaded_list, streamed_list};
    use crate::xfile::{parse_asset_list, tests::zone_head};

    #[test]
    fn finds_lists_and_resolves_names_through_the_ncs() {
        // Table: one NetConstStrings, a rawfile-ish blob we cannot read, three sounds.
        let mut z = zone_head(
            &[None],
            false,
            &[
                (0x34, FOLLOW),
                (0x2C, FOLLOW),
                (0x0E, FOLLOW),
                (0x0E, FOLLOW),
                (0x0E, FOLLOW),
            ],
        );
        let head = parse_asset_list(&z).unwrap();
        let ncs_virt = head.body_virtual;
        z.extend(ncs_bytes("ncs_snd_test", &["music_mainmenu_mp", "ui_x"]));
        // VIRTUAL of the strings: name (13) then table aligned 8.
        let table = (ncs_virt + 13 + 7) & !7;
        let music = table + 16;
        z.extend_from_slice(&[0x5a; 333]); // the unreadable asset
        let first = z.len();
        z.extend(streamed_list(music, 1, 0x0BBA_D000, 0x0097_E0DC, 131_095));
        // An inline-named list anchors the VIRTUAL count from here.
        let pcm = [1u8; 6];
        z.extend(loaded_list("ui_x", 0x40_0000, &pcm));
        // A list sharing the loaded list's sound file (0x40_0000 + 5 -> 8 + 200).
        let shared = 0x40_0000 + 8 + 200;
        let mut l = Vec::new();
        l.extend_from_slice(&FOLLOW.to_le_bytes());
        l.extend_from_slice(&FOLLOW.to_le_bytes());
        l.extend_from_slice(&1u64.to_le_bytes());
        l.extend_from_slice(b"ui_y\0");
        // Its name sits where the loaded list's VIRTUAL data ends: sound file
        // 0x4000d0 + 56, "null" (5), then the insert slot aligned 8.
        let y_name = 0x40_0118;
        l.extend(alias(
            Ptr::Ref {
                block: 3,
                offset: y_name,
            }
            .encode(),
            Ptr::Ref {
                block: 3,
                offset: shared as u32,
            }
            .encode(),
        ));
        z.extend(l);

        let list = parse_asset_list(&z).unwrap();
        let scan = scan_sounds(&z, &list);
        assert_eq!(scan.net_const_strings.len(), 1);
        assert_eq!((scan.expected, scan.lists.len()), (3, 3));
        assert_eq!(scan.lists[0].offset, first);
        assert_eq!(scan.chained, 2);
        assert_eq!(scan.conflicts, 0);
        assert_eq!(scan.anchored, 2);
        let music_list = scan.find("music_mainmenu_mp").unwrap();
        assert_eq!(music_list.aliases[0].name.text(), Some("music_mainmenu_mp"));
        let s = scan.streamed().next().unwrap().1;
        assert_eq!((s.file_index, s.offset), (1, 0x0BBA_D000));
        // The shared sound file resolves to the loaded list's.
        let y = scan.find("ui_y").unwrap();
        let SoundFileRef::Shared {
            resolved: Some(f), ..
        } = &y.aliases[0].sound
        else {
            panic!("shared sound file not resolved: {:?}", y.aliases[0].sound);
        };
        assert!(matches!(f.data, SoundData::Loaded(LoadedRef::Inline(_))));
        assert_eq!(scan.unresolved_names(), 0);
    }

    #[test]
    fn nothing_to_do_without_sounds() {
        let z = zone_head(&[None], false, &[(0x05, FOLLOW)]);
        let list = parse_asset_list(&z).unwrap();
        let scan = scan_sounds(&z, &list);
        assert_eq!((scan.expected, scan.lists.len()), (0, 0));
    }

    #[test]
    fn follow_search() {
        let mut d = vec![0xFFu8; 5];
        d.extend_from_slice(&FOLLOW_BYTES);
        assert_eq!(find_follow(&d, 0), Some(5));
        assert_eq!(find_follow(&d, 6), None);
    }
}
