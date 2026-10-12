//! Sound alias lists (type 0x0E, `snd_alias_list_t`) and what they hold.
//!
//! x64 layouts (sizes measured with an MSVC x64 layout of the zone tools'
//! IW6 headers, and every field below checked on retail bytes):
//! ```text
//! snd_alias_list_t { char* name; snd_alias_t* head; i32 count; }                   24  (TEMP)
//! snd_alias_t                                                                    200
//!   0 name  8 subtitle  16 secondaryAliasName  24 chainAliasName  32 mixerGroup
//!  40 SoundFile* soundFile   48 i32 sequence  52 f32 volMin  56 f32 volMax
//!  60 i32 volModIndex  64 f32 pitchMin  68 f32 pitchMax  72 f32 distMin  76 f32 distMax
//!  80 f32 velocityMin  84 u32 flags  88 u8 masterPriority  92 f32 masterPercentage
//!  96 f32 slavePercentage  100 f32 probability  104 f32 lfePercentage
//! 108 f32 centerPercentage  112 i32 startDelay  120 SndCurve* volumeFalloffCurve
//! 128 SndCurve* lpfCurve  136 SndCurve* reverbSendCurve  144 f32 envelopMin/Max/Percentage
//! 160 SpeakerMap* speakerMap  168 f32 wetMixOverride  176 OcclusionShape* occlusionShape
//! 184 u8 allowDoppler  192 DopplerPreset* dopplerPreset
//! SoundFile { u8 type; u8 exists; SoundFileRef u; }                                56
//!   type 1 loaded:   u.loadSnd (LoadedSound*)
//!   type 2 streamed: u.streamSnd = StreamedSound { StreamFileName filename; u32 totalMsec; }
//!   type 3 primed:   u.primedSnd = { StreamFileName filename; LoadedSound* loadedPart;
//!                                    i32 dataOffset; i32 totalSize; u32 primedCrc; }
//! StreamFileName { u16 isLocalized; u16 fileIndex; union { {char* dir; char* name;} raw;
//!                                                        {u64 offset; u64 length;} packed; } }  24
//! LoadedSound { char* name; XA2Sound { char* data; char* seekTable;
//!               RiffAudioInfo { u32 sampleRate, dataByteCount, numSamples, avgBytesPerSec;
//!                               u16 channels, numBits, blockAlign; u8 format; };
//!               i32 loadedSize; i32 seekTableSize; } }                              56  (TEMP)
//! SndCurve 152 (TEMP)   SpeakerMap 416   OcclusionShape 32   DopplerPreset 32 (TEMP)
//! ```
//! Load order inside one list (the loader's, which the stream follows): the
//! name, the alias array (VIRTUAL, aligned 4), then per alias its strings,
//! sound file (VIRTUAL, aligned 4; a loaded sound's header is TEMP, its name
//! VIRTUAL, its sample data TEMP, its seek table VIRTUAL; an "insert" data
//! pointer takes an 8-byte VIRTUAL slot), curves (TEMP + name), speaker map
//! (VIRTUAL, aligned 4, + name), occlusion shape (VIRTUAL + name) and doppler
//! preset (TEMP + name). Strings and sound files shared with earlier aliases
//! are references instead (see `scan.rs`).

use std::collections::HashMap;

use crate::cursor::{Cursor, ReadError, Result, text};
use crate::ptr::Ptr;

pub const SND_ALIAS_LIST_LEN: usize = 24;
pub const SND_ALIAS_LEN: usize = 200;
pub const SOUND_FILE_LEN: usize = 56;
pub const LOADED_SOUND_LEN: usize = 56;
pub const SND_CURVE_LEN: usize = 152;
pub const SPEAKER_MAP_LEN: usize = 416;
pub const OCCLUSION_SHAPE_LEN: usize = 32;
pub const DOPPLER_PRESET_LEN: usize = 32;
/// The most aliases one list holds (the count is read as a sanity bound).
pub const MAX_ALIASES: u32 = 255;

pub const SAT_UNKNOWN: u8 = 0;
pub const SAT_LOADED: u8 = 1;
pub const SAT_STREAMED: u8 = 2;
pub const SAT_PRIMED: u8 = 3;

/// A string field: its own copy here, a reference to an earlier copy, or none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    None,
    Inline(String),
    /// VIRTUAL offset of the shared string; `resolved` once found.
    Ref {
        virt: u64,
        resolved: Option<String>,
    },
}

impl Name {
    pub fn text(&self) -> Option<&str> {
        match self {
            Name::None => None,
            Name::Inline(s) => Some(s),
            Name::Ref { resolved, .. } => resolved.as_deref(),
        }
    }

    /// The text, or `<ref 0x...>` when the reference was not resolved.
    pub fn display(&self) -> String {
        match self {
            Name::None => String::new(),
            Name::Inline(s) => s.clone(),
            Name::Ref {
                resolved: Some(s), ..
            } => s.clone(),
            Name::Ref {
                virt,
                resolved: None,
            } => format!("<ref {virt:#x}>"),
        }
    }

    pub fn resolve(&mut self, pool: &HashMap<u64, String>) {
        if let Name::Ref { virt, resolved } = self
            && resolved.is_none()
        {
            *resolved = pool.get(virt).cloned();
        }
    }
}

/// A sound in `soundfile<file_index>.pak` (`eng_soundfile<N>.pak` in the
/// language folder when localized): `length` bytes at `offset`, a FLAC stream
/// in the retail paks. `file_index == 0` names a loose file instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamedSound {
    pub localized: bool,
    pub file_index: u16,
    pub offset: u64,
    pub length: u64,
    pub total_msec: u32,
    /// `file_index == 0`: the loose file's folder and name.
    pub raw_dir: Option<Name>,
    pub raw_name: Option<Name>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedSound {
    pub name: Name,
    pub sample_rate: u32,
    pub data_bytes: u32,
    pub samples: u32,
    pub channels: u16,
    pub bits: u16,
    /// `ERiffAudioFormat`: 1 PCM, 3 ADPCM, 5 MP3, 7 FLAC ...
    pub format: u8,
    /// File offset of the sample data in the zone image (when inline).
    pub data_offset: Option<usize>,
    pub data_len: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadedRef {
    Inline(LoadedSound),
    /// A reference to a loaded sound elsewhere (`virt` is the VIRTUAL target).
    Shared(Ptr),
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoundData {
    Unknown,
    Loaded(LoadedRef),
    Streamed(StreamedSound),
    Primed {
        stream: StreamedSound,
        loaded_part: LoadedRef,
        data_offset: i32,
        total_size: i32,
        crc: u32,
    },
}

impl SoundData {
    /// The pak stream, for streamed and primed sounds.
    pub fn stream(&self) -> Option<&StreamedSound> {
        match self {
            SoundData::Streamed(s) | SoundData::Primed { stream: s, .. } => Some(s),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundFile {
    pub exists: bool,
    pub data: SoundData,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoundFileRef {
    None,
    Inline(SoundFile),
    /// Shared with an earlier alias; `resolved` once found.
    Shared {
        virt: u64,
        resolved: Option<SoundFile>,
    },
}

impl SoundFileRef {
    pub fn get(&self) -> Option<&SoundFile> {
        match self {
            SoundFileRef::Inline(f) => Some(f),
            SoundFileRef::Shared { resolved, .. } => resolved.as_ref(),
            SoundFileRef::None => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundAlias {
    pub name: Name,
    pub subtitle: Name,
    pub secondary: Name,
    pub chain: Name,
    pub mixer_group: Name,
    pub sound: SoundFileRef,
    pub sequence: i32,
    pub vol_min: f32,
    pub vol_max: f32,
    pub vol_mod_index: i32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub dist_min: f32,
    pub dist_max: f32,
    pub flags: u32,
    pub probability: f32,
    pub start_delay: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundAliasList {
    /// File offset of the list header in the zone image, and of its end.
    pub offset: usize,
    pub end: usize,
    pub name: Name,
    pub aliases: Vec<SoundAlias>,
}

impl SoundAliasList {
    pub fn name(&self) -> String {
        self.name.display()
    }
}

/// What reading one list learned about VIRTUAL offsets (when known).
#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub strings: Vec<(u64, String)>,
    /// (VIRTUAL offset of an inline SoundFile, alias index)
    pub sound_files: Vec<(u64, usize)>,
    /// Every VIRTUAL reference the list makes (names, sound files), for calibration.
    pub refs: Vec<u64>,
    /// VIRTUAL offset at the start of the list's VIRTUAL data.
    pub start_virt: Option<u64>,
    pub end_virt: Option<u64>,
}

fn ref_or_invalid(p: Ptr, what: &'static str, at: usize) -> Result<Option<u64>> {
    match p {
        Ptr::Null => Ok(None),
        Ptr::Ref { block: 3, offset } => Ok(Some(offset as u64)),
        _ => Err(ReadError::Invalid(what, at)),
    }
}

fn string_field(
    c: &mut Cursor,
    p: Ptr,
    what: &'static str,
    at: usize,
    trace: &mut Trace,
) -> Result<Name> {
    if p.is_inline() {
        let (s, _, v) = c.cstr()?;
        let s = text(s);
        if let Some(v) = v {
            trace.strings.push((v, s.clone()));
        }
        return Ok(Name::Inline(s));
    }
    Ok(match ref_or_invalid(p, what, at)? {
        Some(virt) => {
            trace.refs.push(virt);
            Name::Ref {
                virt,
                resolved: None,
            }
        }
        None => Name::None,
    })
}

fn field_at(c: &mut Cursor, at: usize, what: &'static str, trace: &mut Trace) -> Result<Name> {
    let p = c.ptr_at(at)?;
    string_field(c, p, what, at, trace)
}

fn is_name_text(s: &[u8]) -> bool {
    !s.is_empty() && s.iter().all(|&b| (0x21..0x7f).contains(&b))
}

/// An inline `LoadedSound` (its header first, in TEMP).
fn read_loaded_sound(c: &mut Cursor, trace: &mut Trace) -> Result<LoadedSound> {
    let h = c.other_block(LOADED_SOUND_LEN)?;
    let name_ptr = c.ptr_at(h)?;
    let data_ptr = c.ptr_at(h + 8)?;
    let seek_ptr = c.ptr_at(h + 16)?;
    let info = h + 24;
    let loaded_size = c.u32_at(h + 48)?;
    let seek_size = c.u32_at(h + 52)?;
    let name = string_field(c, name_ptr, "loaded sound name", h, trace)?;
    let mut data_offset = None;
    if data_ptr.is_inline() {
        data_offset = Some(c.other_block(loaded_size as usize)?);
        if data_ptr == Ptr::Insert {
            c.insert_slot();
        }
    }
    if seek_ptr.is_inline() {
        c.virt_bytes(seek_size as usize)?;
        if seek_ptr == Ptr::Insert {
            c.insert_slot();
        }
    }
    Ok(LoadedSound {
        name,
        sample_rate: c.u32_at(info)?,
        data_bytes: c.u32_at(info + 4)?,
        samples: c.u32_at(info + 8)?,
        channels: c.u16_at(info + 16)?,
        bits: c.u16_at(info + 18)?,
        format: c.u8_at(info + 22)?,
        data_offset,
        data_len: loaded_size,
    })
}

fn loaded_ref(c: &mut Cursor, p: Ptr, at: usize, trace: &mut Trace) -> Result<LoadedRef> {
    Ok(match p {
        Ptr::Null => LoadedRef::None,
        p if p.is_inline() => LoadedRef::Inline(read_loaded_sound(c, trace)?),
        p @ Ptr::Ref { .. } => LoadedRef::Shared(p),
        _ => return Err(ReadError::Invalid("loaded sound pointer", at)),
    })
}

fn stream_file_name(c: &mut Cursor, s: usize, trace: &mut Trace) -> Result<StreamedSound> {
    let localized = c.u16_at(s)? != 0;
    let file_index = c.u16_at(s + 2)?;
    let mut out = StreamedSound {
        localized,
        file_index,
        offset: 0,
        length: 0,
        total_msec: 0,
        raw_dir: None,
        raw_name: None,
    };
    if file_index == 0 {
        let dir = c.ptr_at(s + 8)?;
        let name = c.ptr_at(s + 16)?;
        out.raw_dir = Some(string_field(c, dir, "stream dir", s + 8, trace)?);
        out.raw_name = Some(string_field(c, name, "stream name", s + 16, trace)?);
    } else {
        out.offset = c.u64_at(s + 8)?;
        out.length = c.u64_at(s + 16)?;
    }
    Ok(out)
}

/// An inline `SoundFile` (VIRTUAL, aligned 4) and what it holds.
fn read_sound_file(c: &mut Cursor, alias: usize, trace: &mut Trace) -> Result<SoundFile> {
    let (s, v) = c.virt_struct(SOUND_FILE_LEN, 4)?;
    if let Some(v) = v {
        trace.sound_files.push((v, alias));
    }
    let kind = c.u8_at(s)?;
    let exists = c.u8_at(s + 1)? != 0;
    let u = s + 8;
    let data = match kind {
        SAT_UNKNOWN => SoundData::Unknown,
        SAT_LOADED => {
            let p = c.ptr_at(u)?;
            SoundData::Loaded(loaded_ref(c, p, u, trace)?)
        }
        SAT_STREAMED => {
            let mut stream = stream_file_name(c, u, trace)?;
            stream.total_msec = c.u32_at(u + 24)?;
            SoundData::Streamed(stream)
        }
        SAT_PRIMED => {
            let mut stream = stream_file_name(c, u, trace)?;
            let p = c.ptr_at(u + 24)?;
            let loaded_part = loaded_ref(c, p, u + 24, trace)?;
            stream.total_msec = 0;
            SoundData::Primed {
                stream,
                loaded_part,
                data_offset: c.u32_at(u + 32)? as i32,
                total_size: c.u32_at(u + 36)? as i32,
                crc: c.u32_at(u + 40)?,
            }
        }
        _ => return Err(ReadError::Invalid("sound file type", s)),
    };
    Ok(SoundFile { exists, data })
}

/// Skips an inline asset header in TEMP whose name pointer is at `name_off`.
fn skip_temp_named(c: &mut Cursor, len: usize, name_off: usize, trace: &mut Trace) -> Result<()> {
    let h = c.other_block(len)?;
    let p = c.ptr_at(h + name_off)?;
    string_field(c, p, "asset name", h + name_off, trace)?;
    Ok(())
}

fn skip_virt_named(c: &mut Cursor, len: usize, name_off: usize, trace: &mut Trace) -> Result<()> {
    let (h, _) = c.virt_struct(len, 4)?;
    // A built-in default (the retail `OcclusionShape`s) names itself with a
    // raw pointer into the game's executable: no string in the zone.
    match c.ptr_at(h + name_off)? {
        Ptr::Other(_) => {}
        p => {
            string_field(c, p, "struct name", h + name_off, trace)?;
        }
    }
    Ok(())
}

/// Reads one sound alias list at `offset`, whose VIRTUAL data starts at
/// `virt` (if known). Strict: anything that is not a well-formed list is an
/// error, so the scan can try every candidate offset.
pub fn read_sound_alias_list(
    zone: &[u8],
    offset: usize,
    virt: Option<u64>,
) -> Result<(SoundAliasList, Trace)> {
    let mut c = Cursor::new(zone, offset, virt);
    let h = c.other_block(SND_ALIAS_LIST_LEN)?;
    let name_ptr = c.ptr_at(h)?;
    let head_ptr = c.ptr_at(h + 8)?;
    let count = c.u32_at(h + 16)?;
    if head_ptr != Ptr::Follow || count == 0 || count > MAX_ALIASES {
        return Err(ReadError::Invalid("sound alias list header", h));
    }
    let mut trace = Trace {
        start_virt: virt,
        ..Trace::default()
    };
    let name = if name_ptr.is_inline() {
        let (s, _, v) = c.cstr()?;
        if !is_name_text(s) {
            return Err(ReadError::Invalid("sound alias list name", h));
        }
        let s = text(s);
        if let Some(v) = v {
            trace.strings.push((v, s.clone()));
        }
        Name::Inline(s)
    } else {
        match name_ptr {
            Ptr::Ref { block: 3, offset } => {
                trace.refs.push(offset as u64);
                Name::Ref {
                    virt: offset as u64,
                    resolved: None,
                }
            }
            _ => return Err(ReadError::Invalid("sound alias list name pointer", h)),
        }
    };
    let (array, _) = c.virt_struct(SND_ALIAS_LEN * count as usize, 4)?;
    // Every alias names itself with the list's name: the same reference, or
    // (inline list name) a reference to the list's copy.
    for i in 0..count as usize {
        let p = c.ptr_at(array + SND_ALIAS_LEN * i)?;
        let ok = match name_ptr {
            Ptr::Ref { .. } => p == name_ptr || p.is_inline(),
            _ => matches!(p, Ptr::Ref { block: 3, .. }) || p.is_inline(),
        };
        if !ok {
            return Err(ReadError::Invalid(
                "sound alias name",
                array + SND_ALIAS_LEN * i,
            ));
        }
    }
    let mut aliases = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let a = array + SND_ALIAS_LEN * i;
        let alias_name = field_at(&mut c, a, "alias name", &mut trace)?;
        let subtitle = field_at(&mut c, a + 8, "subtitle", &mut trace)?;
        let secondary = field_at(&mut c, a + 16, "secondary alias", &mut trace)?;
        let chain = field_at(&mut c, a + 24, "chain alias", &mut trace)?;
        let mixer_group = field_at(&mut c, a + 32, "mixer group", &mut trace)?;
        let sf = c.ptr_at(a + 40)?;
        let sound = match sf {
            Ptr::Null => SoundFileRef::None,
            p if p.is_inline() => SoundFileRef::Inline(read_sound_file(&mut c, i, &mut trace)?),
            Ptr::Ref { block: 3, offset } => {
                trace.refs.push(offset as u64);
                SoundFileRef::Shared {
                    virt: offset as u64,
                    resolved: None,
                }
            }
            _ => return Err(ReadError::Invalid("sound file pointer", a + 40)),
        };
        for off in [120, 128, 136] {
            let p = c.ptr_at(a + off)?;
            if p.is_inline() {
                skip_temp_named(&mut c, SND_CURVE_LEN, 8, &mut trace)?;
            } else if matches!(p, Ptr::Other(_)) {
                return Err(ReadError::Invalid("sound curve pointer", a + off));
            }
        }
        for (off, len, name_off, temp) in [
            (160, SPEAKER_MAP_LEN, 8, false),
            (176, OCCLUSION_SHAPE_LEN, 8, false),
            (192, DOPPLER_PRESET_LEN, 0, true),
        ] {
            let p = c.ptr_at(a + off)?;
            if p.is_inline() {
                if temp {
                    skip_temp_named(&mut c, len, name_off, &mut trace)?;
                } else {
                    skip_virt_named(&mut c, len, name_off, &mut trace)?;
                }
            } else if matches!(p, Ptr::Other(_)) {
                return Err(ReadError::Invalid("sound alias pointer", a + off));
            }
        }
        aliases.push(SoundAlias {
            name: alias_name,
            subtitle,
            secondary,
            chain,
            mixer_group,
            sound,
            sequence: c.u32_at(a + 48)? as i32,
            vol_min: c.f32_at(a + 52)?,
            vol_max: c.f32_at(a + 56)?,
            vol_mod_index: c.u32_at(a + 60)? as i32,
            pitch_min: c.f32_at(a + 64)?,
            pitch_max: c.f32_at(a + 68)?,
            dist_min: c.f32_at(a + 72)?,
            dist_max: c.f32_at(a + 76)?,
            flags: c.u32_at(a + 84)?,
            probability: c.f32_at(a + 100)?,
            start_delay: c.u32_at(a + 112)? as i32,
        });
    }
    trace.end_virt = c.virt();
    Ok((
        SoundAliasList {
            offset,
            end: c.pos(),
            name,
            aliases,
        },
        trace,
    ))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ptr::{FOLLOW, INSERT};

    fn r(v: u64) -> u64 {
        Ptr::Ref {
            block: 3,
            offset: v as u32,
        }
        .encode()
    }

    /// One alias: (alias name ptr, sound file ptr), the rest zero.
    pub(crate) fn alias(name: u64, sound: u64) -> Vec<u8> {
        let mut a = vec![0u8; SND_ALIAS_LEN];
        a[0..8].copy_from_slice(&name.to_le_bytes());
        a[40..48].copy_from_slice(&sound.to_le_bytes());
        a[52..56].copy_from_slice(&0.5f32.to_le_bytes());
        a
    }

    pub(crate) fn streamed_file(index: u16, offset: u64, length: u64, msec: u32) -> Vec<u8> {
        let mut f = vec![0u8; SOUND_FILE_LEN];
        f[0] = SAT_STREAMED;
        f[1] = 1;
        f[10..12].copy_from_slice(&index.to_le_bytes());
        f[16..24].copy_from_slice(&offset.to_le_bytes());
        f[24..32].copy_from_slice(&length.to_le_bytes());
        f[32..36].copy_from_slice(&msec.to_le_bytes());
        f
    }

    /// A list named by reference with one streamed alias (as code_post_gfx_mp's music).
    pub(crate) fn streamed_list(
        name_virt: u64,
        index: u16,
        offset: u64,
        length: u64,
        msec: u32,
    ) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&r(name_virt).to_le_bytes());
        b.extend_from_slice(&FOLLOW.to_le_bytes());
        b.extend_from_slice(&1u64.to_le_bytes());
        b.extend(alias(r(name_virt), FOLLOW));
        b.extend(streamed_file(index, offset, length, msec));
        b
    }

    /// A list with an inline name and a loaded sound whose data pointer is "insert".
    pub(crate) fn loaded_list(name: &str, name_virt: u64, pcm: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&FOLLOW.to_le_bytes());
        b.extend_from_slice(&FOLLOW.to_le_bytes());
        b.extend_from_slice(&1u64.to_le_bytes());
        b.extend_from_slice(name.as_bytes());
        b.push(0);
        b.extend(alias(r(name_virt), FOLLOW));
        let mut f = vec![0u8; SOUND_FILE_LEN];
        f[0] = SAT_LOADED;
        f[1] = 1;
        f[8..16].copy_from_slice(&FOLLOW.to_le_bytes());
        b.extend(f);
        let mut l = vec![0u8; LOADED_SOUND_LEN];
        l[0..8].copy_from_slice(&FOLLOW.to_le_bytes());
        l[8..16].copy_from_slice(&INSERT.to_le_bytes());
        l[24..28].copy_from_slice(&22050u32.to_le_bytes());
        l[40..42].copy_from_slice(&1u16.to_le_bytes());
        l[42..44].copy_from_slice(&16u16.to_le_bytes());
        l[46] = 1;
        l[48..52].copy_from_slice(&(pcm.len() as u32).to_le_bytes());
        b.extend(l);
        b.extend_from_slice(b"null\0");
        b.extend_from_slice(pcm);
        b
    }

    #[test]
    fn reads_a_streamed_alias() {
        let data = streamed_list(0x1718E, 1, 0x0BBA_D000, 0x0097_E0DC, 131_095);
        let (list, trace) = read_sound_alias_list(&data, 0, None).unwrap();
        assert_eq!(list.end, data.len());
        assert_eq!(
            list.name,
            Name::Ref {
                virt: 0x1718E,
                resolved: None
            }
        );
        let SoundData::Streamed(s) = &list.aliases[0].sound.get().unwrap().data else {
            panic!("not streamed");
        };
        assert_eq!(
            (s.file_index, s.offset, s.length, s.total_msec),
            (1, 0x0BBA_D000, 0x0097_E0DC, 131_095)
        );
        assert!(!s.localized);
        assert_eq!(list.aliases[0].vol_min, 0.5);
        assert!(trace.sound_files.is_empty()); // VIRTUAL unknown
    }

    #[test]
    fn counts_virtual_through_a_loaded_sound() {
        let pcm = [7u8; 10];
        let data = loaded_list("ui_x", 0x1000, &pcm);
        let (list, trace) = read_sound_alias_list(&data, 0, Some(0x1000)).unwrap();
        assert_eq!(list.end, data.len());
        assert_eq!(list.name, Name::Inline("ui_x".into()));
        // name 5 -> 0x1005, align 4 -> 0x1008 + 200 = 0x10d0; sound file 56 -> 0x1108;
        // loaded name "null" 5 -> 0x110d; insert slot: align 8 -> 0x1110 + 8.
        assert_eq!(trace.sound_files, vec![(0x10d0, 0)]);
        assert_eq!(trace.end_virt, Some(0x1118));
        let SoundData::Loaded(LoadedRef::Inline(l)) = &list.aliases[0].sound.get().unwrap().data
        else {
            panic!("not loaded");
        };
        assert_eq!(
            (l.sample_rate, l.channels, l.bits, l.format, l.data_len),
            (22050, 1, 16, 1, 10)
        );
        assert_eq!(l.name, Name::Inline("null".into()));
        assert_eq!(l.data_offset, Some(data.len() - 10));
    }

    #[test]
    fn rejects_lookalikes() {
        let mut data = streamed_list(0x10, 1, 0, 0, 0);
        data[16] = 0; // count 0
        assert!(read_sound_alias_list(&data, 0, None).is_err());
        let mut data = streamed_list(0x10, 1, 0, 0, 0);
        data[24..32].copy_from_slice(&r(0x20).to_le_bytes()); // alias names another string
        assert!(read_sound_alias_list(&data, 0, None).is_err());
        let mut data = streamed_list(0x10, 1, 0, 0, 0);
        data[24 + 200] = 9; // sound file type 9
        assert!(read_sound_alias_list(&data, 0, None).is_err());
    }
}
