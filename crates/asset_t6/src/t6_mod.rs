//! bo2mp: a Black Ops II mod's zone (`mod.ff`, Plutonium's layout) made fit
//! for a match beside Black Ops II's own zones, and the animation files its
//! archive carries (`xanim/<name>`, raw animation files, version 19).
//!
//! A mod zone is a Black Ops II zone: its weapons, attachments, camos,
//! models, materials, effects and sound banks join the match as the shared
//! zones' do. What would change the game rather than add guns is left out
//! (`sanitize_capture`): its string tables (a whole new `mp/statsTable.csv`
//! that renumbers Black Ops II's items, its gun levels), its menu scripts
//! (Lua) and game scripts (GSC source). Its guns get Create-a-Class rows of
//! their own instead (`bo2_profile::guest`).

use crate::capture::{XAnimRef, ZoneCapture};

/// Raw animation files of this version are read (Black Ops' and Black Ops
/// II's `xanim/` files).
pub const RAW_XANIM_VERSION: u16 = 19;

/// Leave out of a mod zone's capture what would change the game rather
/// than add to it; returns what was left out (for the load report).
pub fn sanitize_capture(c: &mut ZoneCapture) -> Vec<String> {
    let mut out = Vec::new();
    let tables: Vec<String> = c.string_tables.iter().map(|t| t.name.clone()).collect();
    if !tables.is_empty() {
        out.push(format!("string tables {}", tables.join(" ")));
    }
    c.string_tables.clear();
    let mut dropped = Vec::new();
    c.raw_files.retain(|(name, _)| {
        let lower = name.to_ascii_lowercase();
        let keep = !(lower.ends_with(".lua")
            || lower.ends_with(".gsc")
            || lower.ends_with(".csc")
            || lower.ends_with(".cfg")
            || lower.starts_with("script:")
            || lower == "mod");
        if !keep {
            dropped.push(name.clone());
        }
        keep
    });
    if !dropped.is_empty() {
        out.push(format!("scripts and raw files {}", dropped.join(" ")));
    }
    out
}

/// The animations the zone's weapons and attachments name that neither the
/// zone nor `have` (the names the other zones carry, lower case) holds:
/// the ones to look for in the mod's archive, lower case, sorted.
#[must_use]
pub fn missing_xanims(c: &ZoneCapture, have: &std::collections::HashSet<String>) -> Vec<String> {
    let inline: std::collections::HashSet<String> = c.xanims.iter().map(|x| x.name.to_ascii_lowercase()).collect();
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let names = c
        .weapons
        .iter()
        .flat_map(|w| w.xanims.iter())
        .chain(c.attachment_uniques.iter().flat_map(|u| u.xanims.iter()));
    for n in names {
        if n.is_empty() || n.starts_with(',') {
            continue;
        }
        let n = n.to_ascii_lowercase();
        if !inline.contains(&n) && !have.contains(&n) {
            out.insert(n);
        }
    }
    out.into_iter().collect()
}

/// Read a raw animation file (`xanim/<name>`, version 19) into the packed
/// form a zone's animations have (Black Ops II's part types, as the clip
/// decoder reads them).
///
/// The file: version (u16), frame count (u16), bone count (u16), flags
/// (u8: 1 looping, 2 delta), asset type (u8), frame rate (u16); per bone a
/// bit of "negative w" and a bit of "turns about z only"; the bone names;
/// then per bone its rotation keys (count u16; frame numbers, bytes or
/// u16s, unless one key or every frame; then x y z, or z alone, as i16
/// over 32767) and its translation keys (count u16; frames as above; one
/// key: x y z as floats; more: a byte that says bytes or u16s, the
/// minimum and range as floats, then each key's x y z over 255 or 65535);
/// last the notetracks (count u8; name, frame u16).
pub fn raw_xanim(name: &str, bytes: &[u8]) -> Result<XAnimRef, String> {
    let mut r = Reader { b: bytes, at: 0, name };
    let version = r.u16()?;
    if version != RAW_XANIM_VERSION {
        return Err(format!("xanim {name}: version {version}, not {RAW_XANIM_VERSION}"));
    }
    let frames = r.u16()?;
    let bones = usize::from(r.u16()?);
    let flags = r.u8()?;
    let _asset_type = r.u8()?;
    let framerate = r.u16()?;
    if flags & 2 != 0 {
        return Err(format!("xanim {name}: delta animations are not read"));
    }
    let looping = flags & 1 != 0;
    // Frames 0..frames, and the loop's end again when it loops.
    let full = usize::from(frames) + usize::from(looping);
    let numframes = u16::try_from(full.saturating_sub(1)).map_err(|_| format!("xanim {name}: too many frames"))?;
    let bits = if bones == 0 { 0 } else { (bones - 1) / 8 + 1 };
    let flip = r.take(bits)?.to_vec();
    let simple = r.take(bits)?.to_vec();
    let bit = |set: &[u8], i: usize| (set[i >> 3] >> (i & 7)) & 1 != 0;
    let mut names = Vec::with_capacity(bones);
    for _ in 0..bones {
        names.push(r.cstr()?);
    }
    let key_frames = |r: &mut Reader<'_>, n: usize| -> Result<Vec<u16>, String> {
        if n == 1 {
            return Ok(vec![0]);
        }
        if n == full {
            return Ok((0..n as u16).collect());
        }
        if full <= 256 {
            Ok(r.take(n)?.iter().map(|&b| u16::from(b)).collect())
        } else {
            (0..n).map(|_| r.u16()).collect()
        }
    };
    let mut tracks: Vec<Track> = Vec::with_capacity(bones);
    for (i, bone) in names.iter().enumerate() {
        let (neg, half) = (bit(&flip, i), bit(&simple, i));
        let n = usize::from(r.u16()?);
        let rot_frames = if n > 0 { key_frames(&mut r, n)? } else { Vec::new() };
        let mut rot = Vec::with_capacity(n);
        for _ in 0..n {
            rot.push(if half {
                let z = r.i16()?;
                vec![z, w_of(&[z], neg)]
            } else {
                let (x, y, z) = (r.i16()?, r.i16()?, r.i16()?);
                vec![x, y, z, w_of(&[x, y, z], neg)]
            });
        }
        let t = usize::from(r.u16()?);
        let translation = match t {
            0 => Trans::None,
            1 => Trans::Constant([r.f32()?, r.f32()?, r.f32()?]),
            _ => {
                let frames = key_frames(&mut r, t)?;
                let small = r.u8()? != 0;
                let mins = [r.f32()?, r.f32()?, r.f32()?];
                let range = [r.f32()?, r.f32()?, r.f32()?];
                let mut keys = Vec::with_capacity(t * 3);
                for _ in 0..t * 3 {
                    keys.push(if small { u16::from(r.u8()?) } else { r.u16()? });
                }
                Trans::Keyed { frames, small, mins, range, keys }
            }
        };
        tracks.push(Track { name: bone.clone(), half, rot_frames, rot, translation });
    }
    let notes = usize::from(r.u8()?);
    let mut notifies = Vec::with_capacity(notes);
    for _ in 0..notes {
        let note = r.cstr()?;
        let frame = r.u16()?;
        let time = if numframes == 0 { 0.0 } else { (f32::from(frame) / f32::from(numframes)).clamp(0.0, 1.0) };
        notifies.push((note, time));
    }
    if r.at != bytes.len() {
        return Err(format!("xanim {name}: {} bytes after the notetracks", bytes.len() - r.at));
    }
    pack(name, numframes, f32::from(framerate), looping, tracks, notifies)
}

/// w of a unit quaternion stored without it (x y z, or z alone), as i16
/// over 32767; negative for a bone marked so.
fn w_of(rest: &[i16], negative: bool) -> i16 {
    let sum: f32 = rest.iter().map(|&v| (f32::from(v) / 32767.0).powi(2)).sum();
    let w = ((1.0 - sum).max(0.0).sqrt() * 32767.0).round() as i16;
    if negative { -w } else { w }
}

enum Trans {
    None,
    Constant([f32; 3]),
    Keyed { frames: Vec<u16>, small: bool, mins: [f32; 3], range: [f32; 3], keys: Vec<u16> },
}

struct Track {
    name: String,
    half: bool,
    rot_frames: Vec<u16>,
    /// Each key's stored components: z w (half) or x y z w.
    rot: Vec<Vec<i16>>,
    translation: Trans,
}

/// The packed streams, bones in part-type order: no rotation, keyed half
/// and full rotations, constant half and full rotations; translations by
/// bone index (keyed in bytes, keyed in u16s, constant, none).
fn pack(
    name: &str,
    numframes: u16,
    framerate: f32,
    looping: bool,
    mut tracks: Vec<Track>,
    notifies: Vec<(String, f32)>,
) -> Result<XAnimRef, String> {
    let kind = |t: &Track| match (t.rot.len(), t.half) {
        (0, _) => 0,
        (1, true) => 3,
        (1, false) => 4,
        (_, true) => 1,
        (_, false) => 2,
    };
    tracks.sort_by_key(|t| kind(t));
    if tracks.len() > 255 {
        return Err(format!("xanim {name}: {} bones", tracks.len()));
    }
    let mut bone_count = [0u8; 10];
    for t in &tracks {
        bone_count[kind(t)] += 1;
    }
    let byte_indices = numframes < 256;
    let mut a = XAnimRef {
        name: name.to_owned(),
        numframes,
        framerate,
        looping,
        delta: false,
        delta_trans: Vec::new(),
        delta_quat: Vec::new(),
        bone_count: [0; 10],
        names: tracks.iter().map(|t| t.name.clone()).collect(),
        notifies,
        data_byte: Vec::new(),
        data_short: Vec::new(),
        data_int: Vec::new(),
        random_data_byte: Vec::new(),
        random_data_short: Vec::new(),
        random_data_int: Vec::new(),
        indices: Vec::new(),
    };
    // Rotations, in part-type order (the sort above).
    for t in &tracks {
        match kind(t) {
            1 | 2 => {
                put_indices(&mut a, &t.rot_frames, byte_indices);
                for key in &t.rot {
                    a.random_data_short.extend(key.iter().map(|&v| v as u16));
                }
            }
            3 | 4 => a.data_short.extend(t.rot[0].iter().map(|&v| v as u16)),
            _ => {}
        }
    }
    // Translations, by part type, each with its bone's index.
    let index_of = |t: &Track| tracks.iter().position(|o| std::ptr::eq(o, t)).unwrap_or(0) as u8;
    for (part, wanted) in [(5usize, 0u8), (6, 1), (7, 2), (8, 3)] {
        for t in &tracks {
            let this = match &t.translation {
                Trans::Keyed { small: true, .. } => 0,
                Trans::Keyed { small: false, .. } => 1,
                Trans::Constant(_) => 2,
                Trans::None => 3,
            };
            if this != wanted {
                continue;
            }
            bone_count[part] += 1;
            a.data_byte.push(index_of(t));
            match &t.translation {
                Trans::Keyed { frames, small, mins, range, keys } => {
                    let den = if *small { 255.0 } else { 65535.0 };
                    // (Size, then the mins and step, then the frames: the
                    // pools are apart, so size and frames follow each
                    // other in theirs.)
                    put_indices(&mut a, frames, byte_indices);
                    a.data_int.extend(mins.iter().map(|v| v.to_bits()));
                    a.data_int.extend(range.iter().map(|v| (v / den).to_bits()));
                    if *small {
                        a.random_data_byte.extend(keys.iter().map(|&k| k as u8));
                    } else {
                        a.random_data_short.extend_from_slice(keys);
                    }
                }
                Trans::Constant(v) => a.data_int.extend(v.iter().map(|x| x.to_bits())),
                Trans::None => {}
            }
        }
    }
    bone_count[9] = tracks.len() as u8;
    a.bone_count = bone_count;
    Ok(a)
}

/// A keyed part's size (keys - 1) and its frame numbers: bytes when the
/// animation has fewer than 256 frames, else u16s, from the index pool
/// (with the checkpoints the engine searches by) when there are 65 or more.
fn put_indices(a: &mut XAnimRef, frames: &[u16], byte_indices: bool) {
    let size = frames.len() - 1;
    a.data_short.push(size as u16);
    if byte_indices {
        a.data_byte.extend(frames.iter().map(|&f| f as u8));
    } else if size >= 64 {
        a.indices.extend_from_slice(frames);
        let checkpoints = (frames.len() - 2) / 256 + 2;
        a.data_short.extend((0..checkpoints).map(|k| frames[(k * 256).min(size)]));
    } else {
        a.data_short.extend_from_slice(frames);
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    name: &'a str,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.b.len()).ok_or_else(|| format!("xanim {}: ends early", self.name))?;
        let out = &self.b[self.at..end];
        self.at = end;
        Ok(out)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn i16(&mut self) -> Result<i16, String> {
        Ok(self.u16()? as i16)
    }
    fn f32(&mut self) -> Result<f32, String> {
        let b = self.take(4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn cstr(&mut self) -> Result<String, String> {
        let rest = &self.b[self.at..];
        let end = rest.iter().position(|&c| c == 0).ok_or_else(|| format!("xanim {}: unterminated name", self.name))?;
        let s = String::from_utf8_lossy(&rest[..end]).into_owned();
        self.at += end + 1;
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A raw file: (bones: name, negative w, z only, rotation keys
    /// (frame, components), translation), loop flag, frame count.
    fn raw(frames: u16, looping: bool, bones: &[(&str, bool, bool, Vec<(u8, Vec<i16>)>, Vec<u8>)], notes: &[(&str, u16)]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(19u16.to_le_bytes());
        b.extend(frames.to_le_bytes());
        b.extend((bones.len() as u16).to_le_bytes());
        b.push(u8::from(looping));
        b.push(1);
        b.extend(30u16.to_le_bytes());
        let bits = (bones.len() - 1) / 8 + 1;
        let mut flip = vec![0u8; bits];
        let mut simple = vec![0u8; bits];
        for (i, (_, neg, half, _, _)) in bones.iter().enumerate() {
            if *neg {
                flip[i / 8] |= 1 << (i % 8);
            }
            if *half {
                simple[i / 8] |= 1 << (i % 8);
            }
        }
        b.extend(&flip);
        b.extend(&simple);
        for (name, ..) in bones {
            b.extend(name.as_bytes());
            b.push(0);
        }
        let full = usize::from(frames) + usize::from(looping);
        for (_, _, _, keys, trans) in bones {
            b.extend((keys.len() as u16).to_le_bytes());
            if keys.len() > 1 && keys.len() != full {
                b.extend(keys.iter().map(|(f, _)| *f));
            }
            for (_, v) in keys {
                for c in v {
                    b.extend(c.to_le_bytes());
                }
            }
            b.extend(trans);
        }
        b.push(notes.len() as u8);
        for (n, f) in notes {
            b.extend(n.as_bytes());
            b.push(0);
            b.extend(f.to_le_bytes());
        }
        b
    }

    fn one_trans(v: [f32; 3]) -> Vec<u8> {
        let mut t = 1u16.to_le_bytes().to_vec();
        for x in v {
            t.extend(x.to_le_bytes());
        }
        t
    }

    #[test]
    fn a_raw_animation_packs_as_a_zone_animation_does() {
        // Three frames, not looping: a still root, an arm keyed on frames 0
        // and 2 (full rotation), a gun turning about z on every frame and
        // moving in small steps, and one notetrack.
        let mut small = 3u16.to_le_bytes().to_vec();
        small.push(1);
        for v in [1.0f32, 2.0, 3.0, 0.0, 2.55, 0.0] {
            small.extend(v.to_le_bytes());
        }
        small.extend([0, 0, 0, 0, 100, 0, 0, 255, 0]);
        let bytes = raw(
            3,
            false,
            &[
                ("tag_torso", false, true, vec![], one_trans([0.0, 0.0, 0.0])),
                ("j_shoulder_le", true, false, vec![(0, vec![1000, 0, 0]), (2, vec![2000, 0, 0])], one_trans([4.0, 4.5, -4.25])),
                ("j_gun", false, true, vec![(0, vec![0]), (1, vec![100]), (2, vec![200])], small),
                ("tag_flash", false, false, vec![(0, vec![0, 0, 0])], 0u16.to_le_bytes().to_vec()),
            ],
            &[("fire", 1)],
        );
        let a = raw_xanim("viewmodel_test_fire", &bytes).expect("reads");
        assert_eq!(a.numframes, 2);
        assert!(!a.looping);
        // Part types: no rotation (torso), keyed half (gun), keyed full
        // (shoulder), constant full (flash); translations: one small
        // keyed, two constant, one none.
        assert_eq!(a.bone_count, [1, 1, 1, 0, 1, 1, 0, 2, 1, 4]);
        assert_eq!(a.names, ["tag_torso", "j_gun", "j_shoulder_le", "tag_flash"]);
        assert_eq!(a.notifies, vec![("fire".to_owned(), 0.5)]);
        // The shoulder's w is negative (its bit), the gun's positive.
        let w = |v: u16| v as i16;
        assert!(w(a.random_data_short[3]) > 32000, "gun key 0: z 0, w 1");
        assert!(w(a.random_data_short[2 * 3 + 3]) < -32000, "shoulder key 0 w");

        let parts = xmodel_runtime::RawXAnimParts {
            name: a.name.clone(),
            data_byte: a.data_byte.clone(),
            data_short: a.data_short.clone(),
            data_int: a.data_int.clone(),
            random_data_byte: a.random_data_byte.clone(),
            random_data_short: a.random_data_short.clone(),
            random_data_int: a.random_data_int.clone(),
            numframes: a.numframes,
            flags: u8::from(a.looping),
            bone_count: a.bone_count,
            framerate: a.framerate,
            names: a.names.clone(),
            notifies: Vec::new(),
            indices: a.indices.clone(),
            delta_trans: None,
        };
        let clip = xmodel_runtime::AnimClip::from_parts(&parts).expect("the decoder reads every stream to its end");
        let gun = clip.tracks.iter().position(|t| t.name == "j_gun").unwrap();
        let mid = clip.sample_track(gun, 1.0 / 30.0).unwrap();
        let t = mid.translation.unwrap();
        assert!((t[0] - 1.0).abs() < 1e-4 && (t[1] - 3.0).abs() < 0.02 && (t[2] - 3.0).abs() < 1e-4, "{t:?}");
        let q = mid.rotation.unwrap();
        assert!((q[2] - 100.0 / 32767.0).abs() < 1e-3, "{q:?}");
        let shoulder = clip.tracks.iter().position(|t| t.name == "j_shoulder_le").unwrap();
        assert_eq!(clip.sample_track(shoulder, 0.0).unwrap().translation, Some([4.0, 4.5, -4.25]));
    }

    #[test]
    fn looping_files_key_the_loop_end_and_long_ones_use_the_index_pool() {
        // 300 frames, looping: 301 keys is every frame (no frame numbers);
        // 70 keys name theirs as u16s.
        let every: Vec<(u8, Vec<i16>)> = (0..301).map(|_| (0, vec![0, 0, 0])).collect();
        let mut bytes = raw(300, true, &[("j_a", false, false, every, 0u16.to_le_bytes().to_vec())], &[]);
        assert!(raw_xanim("a", &bytes).is_ok_and(|a| a.numframes == 300 && a.looping));
        // Hand-build a sparse 70-key rotation on a long animation.
        bytes.truncate(bytes.len() - (2 + 301 * 6 + 2 + 1));
        bytes.extend(70u16.to_le_bytes());
        for k in 0..70u16 {
            bytes.extend((k * 4).to_le_bytes());
        }
        for _ in 0..70 {
            bytes.extend([0u8; 6]);
        }
        bytes.extend(0u16.to_le_bytes());
        bytes.push(0);
        let a = raw_xanim("b", &bytes).expect("reads");
        assert_eq!(a.indices.len(), 70);
        let parts = xmodel_runtime::RawXAnimParts {
            name: "b".into(),
            data_byte: a.data_byte,
            data_short: a.data_short,
            data_int: a.data_int,
            random_data_byte: a.random_data_byte,
            random_data_short: a.random_data_short,
            random_data_int: a.random_data_int,
            numframes: a.numframes,
            flags: 1,
            bone_count: a.bone_count,
            framerate: a.framerate,
            names: a.names,
            notifies: Vec::new(),
            indices: a.indices,
            delta_trans: None,
        };
        xmodel_runtime::AnimClip::from_parts(&parts).expect("decodes");
    }

    #[test]
    fn a_wrong_version_or_trailing_bytes_is_refused() {
        let mut bytes = raw(1, false, &[("j_a", false, false, vec![], 0u16.to_le_bytes().to_vec())], &[]);
        assert!(raw_xanim("a", &bytes).is_ok());
        bytes.push(0);
        assert!(raw_xanim("a", &bytes).is_err());
        bytes.pop();
        bytes[0] = 17;
        assert!(raw_xanim("a", &bytes).is_err());
    }

    /// Every raw file in `T6_MOD_XANIM_DIR` (a mod archive's `xanim/`,
    /// extracted) reads and decodes whole. Skipped when it is not set.
    #[test]
    fn a_mod_archive_animations_decode() {
        let Ok(dir) = std::env::var("T6_MOD_XANIM_DIR") else {
            return;
        };
        let mut ok = 0;
        let mut bad = Vec::new();
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let bytes = std::fs::read(e.path()).unwrap();
            let decoded = raw_xanim(&name, &bytes).and_then(|a| {
                let parts = xmodel_runtime::RawXAnimParts {
                    name: name.clone(),
                    data_byte: a.data_byte,
                    data_short: a.data_short,
                    data_int: a.data_int,
                    random_data_byte: a.random_data_byte,
                    random_data_short: a.random_data_short,
                    random_data_int: a.random_data_int,
                    numframes: a.numframes,
                    flags: u8::from(a.looping),
                    bone_count: a.bone_count,
                    framerate: a.framerate,
                    names: a.names,
                    notifies: Vec::new(),
                    indices: a.indices,
                    delta_trans: None,
                };
                xmodel_runtime::AnimClip::from_parts(&parts).map_err(|e| format!("{e:?}"))
            });
            match decoded {
                Ok(_) => ok += 1,
                Err(e) => bad.push(format!("{name}: {e}")),
            }
        }
        eprintln!("{ok} decoded, {} not", bad.len());
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
}
