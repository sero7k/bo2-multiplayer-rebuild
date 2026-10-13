//! Sound alias lists and loaded sounds (x64-zt's S1 layouts and order).
//!
//! An alias's `SoundFile` is loaded once and shared by later aliases by its
//! VIRTUAL offset; a loaded sound is a sub-asset (references name the field
//! that loaded it). AW's loaded sounds keep their samples in VIRTUAL.

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::ptr::Ptr;
use crate::stream::{TEMP, VIRTUAL};

const SAT_LOADED: u8 = 1;
const SAT_STREAMED: u8 = 2;
const SAT_PRIMED: u8 = 3;

/// A sound alias list: its name and aliases.
#[derive(Clone, Debug, Default)]
pub struct SoundAliasList {
    pub name: String,
    pub aliases: Vec<SoundAlias>,
}

#[derive(Clone, Debug, Default)]
pub struct SoundAlias {
    pub name: String,
    pub secondary: Option<String>,
    pub chain: Option<String>,
    /// The `SoundFile` (by its VIRTUAL offset in `Capture::sound_files`).
    pub file: Option<u64>,
    pub vol_min: f32,
    pub vol_max: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub dist_min: f32,
    pub dist_max: f32,
    pub flags: u32,
    pub probability: f32,
}

#[derive(Clone, Debug, Default)]
pub struct LoadedSound {
    pub name: String,
    pub sample_rate: u32,
    pub data_bytes: u32,
    pub samples: u32,
    pub channels: u8,
    pub bits: u8,
    /// 1 PCM, 6 FLAC.
    pub format: u16,
    /// The samples in the zone image (file offset, length), when inline.
    pub data: Option<(usize, usize)>,
    /// `soundfile<N>.pak` range when the samples are streamed.
    pub stream: Option<(u16, u64, u64)>,
}

/// A `SoundFile`: what an alias plays.
#[derive(Clone, Debug, Default)]
pub struct SoundFile {
    pub kind: u8,
    pub loaded: Option<AssetId>,
    /// Streamed/primed: (pak index, offset, length).
    pub stream: Option<(u16, u64, u64)>,
}

fn name_of(w: &mut Walker, field: usize) -> Result<Option<String>> {
    w.xstr(field)
}

/// A `StreamFileName` at `s`: loose files name their folder and file,
/// packed ones a pak range.
fn stream_name(w: &mut Walker, s: usize) -> Result<Option<(u16, u64, u64)>> {
    use L::StreamFileName as N;
    let index = w.s.u16_at(s + N::fileIndex)?;
    if index == 0 {
        name_of(w, s + N::info + L::StreamFileInfo::raw__dir)?;
        name_of(w, s + N::info + L::StreamFileInfo::raw__name)?;
        Ok(None)
    } else {
        Ok(Some((
            index,
            w.s.u64_at(s + N::info + L::StreamFileInfo::packed__offset)?,
            w.s.u64_at(s + N::info + L::StreamFileInfo::packed__length)?,
        )))
    }
}

fn loaded_body(w: &mut Walker, h: usize) -> Result<LoadedSound> {
    use L::LoadedSound as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    let stream = stream_name(w, h + S::filename)?;
    let size = w.s.i32_at(h + S::info__loadedSize)?.max(0) as usize;
    let data = w.blob(h + S::info__data, size, 0)?.map(|at| (at, size));
    w.s.pop();
    Ok(LoadedSound {
        name,
        sample_rate: w.s.u32_at(h + S::info__sampleRate)?,
        data_bytes: w.s.u32_at(h + S::info__dataByteCount)?,
        samples: w.s.u32_at(h + S::info__numSamples)?,
        channels: w.s.u8_at(h + S::info__channels)?,
        bits: w.s.u8_at(h + S::info__numBits)?,
        format: w.s.u16_at(h + S::info__format)?,
        data,
        stream,
    })
}

pub(super) fn loaded_sound(w: &mut Walker, h: usize) -> Result<AssetId> {
    let rec = loaded_body(w, h)?;
    w.cap.loaded.push(rec);
    Ok(AssetId {
        ty: AssetType::LoadedSound,
        index: (w.cap.loaded.len() - 1) as u32,
    })
}

/// The `SoundFile` an alias's field points at; its VIRTUAL offset.
fn sound_file(w: &mut Walker, field: usize) -> Result<Option<u64>> {
    match w.s.ptr_at(field)? {
        Ptr::Ref {
            block: VIRTUAL,
            offset,
        } => Ok(Some(offset as u64)),
        p @ (Ptr::Follow | Ptr::Insert) => {
            if p == Ptr::Insert {
                w.s.insert_slot();
            }
            use L::SoundFile as F;
            let s = w.st(F::SIZE, 3)?;
            let kind = w.s.u8_at(s + F::r#type)?;
            let mut file = SoundFile {
                kind,
                ..Default::default()
            };
            match kind {
                SAT_LOADED => {
                    file.loaded = w.asset(AssetType::LoadedSound, s + F::u__loadSnd)?;
                }
                SAT_STREAMED => {
                    file.stream = stream_name(w, s + F::u__streamSnd__filename)?;
                }
                SAT_PRIMED => {
                    file.loaded = w.asset(AssetType::LoadedSound, s + F::u__primedSnd__loadedPart)?;
                    file.stream = stream_name(w, s + F::u__primedSnd__streamedPart)?;
                }
                _ => {}
            }
            let v = w.vmap.virt_of(s);
            if let Some(v) = v {
                w.cap.sound_files.insert(v, file);
            }
            Ok(v)
        }
        _ => Ok(None),
    }
}

pub(super) fn sound(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::snd_alias_list_t as H;
    use L::snd_alias_t as A;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + H::name)?.unwrap_or_default();
    let count = w.s.u8_at(h + H::count)? as usize;
    let mut aliases = Vec::new();
    if let Some(array) = w.inline_array(h + H::head, A::SIZE * count, 3)? {
        for i in 0..count {
            let a = array + A::SIZE * i;
            // (aliasName is the list's own name pointer.)
            let alias_name = w.xstr(a + A::aliasName)?.unwrap_or_else(|| name.clone());
            w.xstr(a + A::subtitle)?;
            let secondary = w.xstr(a + A::secondaryAliasName)?;
            let chain = w.xstr(a + A::chainAliasName)?;
            let file = sound_file(w, a + A::soundFile)?;
            w.xstr(a + A::squelchName)?;
            w.asset(AssetType::SoundContext, a + A::sndContext)?;
            w.asset(AssetType::SoundCurve, a + A::volumeFalloffCurve)?;
            w.asset(AssetType::LpfCurve, a + A::lpfCurve)?;
            w.asset(AssetType::ReverbCurve, a + A::reverbSendCurve)?;
            if let Some(map) = w.inline_array(a + A::speakerMap, L::SpeakerMap::SIZE, 3)? {
                w.xstr(map + L::SpeakerMap::name)?;
            }
            w.asset(AssetType::DopplerPreset, a + A::dopplerPreset)?;
            aliases.push(SoundAlias {
                name: alias_name,
                secondary,
                chain,
                file,
                vol_min: w.s.f32_at(a + A::volMin)?,
                vol_max: w.s.f32_at(a + A::volMax)?,
                pitch_min: w.s.f32_at(a + A::pitchMin)?,
                pitch_max: w.s.f32_at(a + A::pitchMax)?,
                dist_min: w.s.f32_at(a + A::distMin)?,
                dist_max: w.s.f32_at(a + A::distMax)?,
                flags: w.s.u32_at(a + A::flags)?,
                probability: w.s.f32_at(a + A::probability)?,
            });
        }
    }
    let contexts = w.s.u8_at(h + H::contextListCount)? as usize;
    w.blob(h + H::contextList, 2 * contexts, 1)?;
    w.s.pop();
    w.cap.sounds.push(SoundAliasList { name, aliases });
    Ok(AssetId {
        ty: AssetType::Sound,
        index: (w.cap.sounds.len() - 1) as u32,
    })
}

/// `SndContext` { name; priority } (16 bytes).
pub(super) fn sound_context(w: &mut Walker, h: usize) -> Result<AssetId> {
    let _ = TEMP;
    super::misc::named_only(w, AssetType::SoundContext, h, 0)
}

/// `SoundSubmix` (no x64-zt type; read on retail common_mp): a 24-byte
/// header { name; entries; u32 count } and `count` 24-byte entries
/// { name; u32 id; f32 value; 8 more bytes }.
pub(super) fn submix(w: &mut Walker, h: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = w.xstr(h)?.unwrap_or_default();
    let n = w.s.u32_at(h + 16)? as usize;
    if let Some(at) = w.inline_array(h + 8, 24 * n, 3)? {
        for i in 0..n {
            w.xstr(at + 24 * i)?;
        }
    }
    w.s.pop();
    Ok(w.name_only(AssetType::SoundSubmix, name))
}
