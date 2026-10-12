//! Sound alias lists and loaded sounds, read exactly (see `crate::sound`
//! for the layouts; this is the same order on the block-aware stream).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::ptr::Ptr;
use crate::sound::{
    LoadedRef, LoadedSound, Name, OCCLUSION_SHAPE_LEN, SAT_LOADED, SAT_PRIMED, SAT_STREAMED,
    SND_ALIAS_LEN, SOUND_FILE_LEN, SPEAKER_MAP_LEN, SoundAlias, SoundAliasList, SoundData, SoundFile,
    SoundFileRef, StreamedSound,
};
use crate::stream::{TEMP, VIRTUAL};

fn name_field(w: &mut Walker, field: usize) -> Result<Name> {
    Ok(match w.s.ptr_at(field)? {
        Ptr::Null | Ptr::Other(_) => Name::None,
        Ptr::Ref { block: VIRTUAL, offset } => Name::Ref { virt: offset as u64, resolved: w.string_at(offset as u64) },
        Ptr::Follow | Ptr::Insert => w.xstr(field)?.map_or(Name::None, Name::Inline),
        _ => Name::None,
    })
}

/// A `LoadedSound` whose header is at `h` (TEMP, already read).
fn loaded_body(w: &mut Walker, h: usize) -> Result<LoadedSound> {
    w.s.push(VIRTUAL);
    let name = name_field(w, h)?;
    let size = w.s.u32_at(h + 48)?;
    let seek = w.s.u32_at(h + 52)?;
    w.s.push(TEMP);
    let data = w.blob(h + 8, size as usize, 0)?;
    w.s.pop();
    w.blob(h + 16, seek as usize, 0)?;
    w.s.pop();
    let info = h + 24;
    Ok(LoadedSound {
        name,
        sample_rate: w.s.u32_at(info)?,
        data_bytes: w.s.u32_at(info + 4)?,
        samples: w.s.u32_at(info + 8)?,
        channels: w.s.u16_at(info + 16)?,
        bits: w.s.u16_at(info + 18)?,
        format: w.s.u8_at(info + 22)?,
        data_offset: data,
        data_len: size,
    })
}

pub(super) fn loaded_sound(w: &mut Walker, h: usize) -> Result<AssetId> {
    let rec = loaded_body(w, h)?;
    let name = rec.name.display();
    Ok(w.name_only(AssetType::LoadedSound, name))
}

fn loaded_ref(w: &mut Walker, field: usize) -> Result<LoadedRef> {
    Ok(match w.s.ptr_at(field)? {
        Ptr::Null => LoadedRef::None,
        p @ (Ptr::Follow | Ptr::Insert) => {
            if p == Ptr::Insert {
                w.s.insert_slot();
            }
            w.s.push(TEMP);
            let h = w.st(crate::sound::LOADED_SOUND_LEN, 7)?;
            let rec = loaded_body(w, h);
            w.s.pop();
            let rec = rec?;
            // References to this loaded sound name the field that loaded it.
            if let Some(v) = w.vmap.virt_of(field) {
                let id = w.name_only(AssetType::LoadedSound, rec.name.display());
                w.slots.insert(v, id);
            }
            LoadedRef::Inline(rec)
        }
        p => LoadedRef::Shared(p),
    })
}

fn stream_name(w: &mut Walker, s: usize) -> Result<StreamedSound> {
    let file_index = w.s.u16_at(s + 2)?;
    let mut out = StreamedSound {
        localized: w.s.u16_at(s)? != 0,
        file_index,
        offset: 0,
        length: 0,
        total_msec: 0,
        raw_dir: None,
        raw_name: None,
    };
    if file_index == 0 {
        out.raw_dir = Some(name_field(w, s + 8)?);
        out.raw_name = Some(name_field(w, s + 16)?);
    } else {
        out.offset = w.s.u64_at(s + 8)?;
        out.length = w.s.u64_at(s + 16)?;
    }
    Ok(out)
}

fn sound_file(w: &mut Walker, field: usize) -> Result<SoundFileRef> {
    match w.s.ptr_at(field)? {
        Ptr::Null => Ok(SoundFileRef::None),
        Ptr::Ref { block: VIRTUAL, offset } => Ok(SoundFileRef::Shared { virt: offset as u64, resolved: None }),
        p @ (Ptr::Follow | Ptr::Insert) => {
            if p == Ptr::Insert {
                w.s.insert_slot();
            }
            let s = w.st(SOUND_FILE_LEN, 3)?;
            let kind = w.s.u8_at(s)?;
            let u = s + 8;
            let data = match kind {
                SAT_LOADED => SoundData::Loaded(loaded_ref(w, u)?),
                SAT_STREAMED => {
                    let mut st = stream_name(w, u)?;
                    st.total_msec = w.s.u32_at(u + 24)?;
                    SoundData::Streamed(st)
                }
                SAT_PRIMED => {
                    let stream = stream_name(w, u)?;
                    let loaded_part = loaded_ref(w, u + 24)?;
                    SoundData::Primed {
                        stream,
                        loaded_part,
                        data_offset: w.s.i32_at(u + 32)?,
                        total_size: w.s.i32_at(u + 36)?,
                        crc: w.s.u32_at(u + 40)?,
                    }
                }
                _ => SoundData::Unknown,
            };
            Ok(SoundFileRef::Inline(SoundFile { exists: w.s.u8_at(s + 1)? != 0, data }))
        }
        _ => Ok(SoundFileRef::None),
    }
}

/// A sub-asset or struct with a name at `name_off` (curves and presets:
/// header in TEMP; speaker maps and occlusion shapes in VIRTUAL).
fn named_struct(w: &mut Walker, field: usize, len: usize, name_off: usize, temp: bool) -> Result<()> {
    let p = w.s.ptr_at(field)?;
    if !p.is_inline() {
        return Ok(());
    }
    if p == Ptr::Insert {
        w.s.insert_slot();
    }
    if temp {
        w.s.push(TEMP);
    }
    let at = w.st(len, 3)?;
    if temp {
        w.s.push(VIRTUAL);
    }
    if !matches!(w.s.ptr_at(at + name_off)?, Ptr::Other(_)) {
        w.xstr(at + name_off)?;
    }
    if temp {
        w.s.pop();
        w.s.pop();
    }
    Ok(())
}

pub(super) fn sound(w: &mut Walker, h: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = name_field(w, h)?;
    let count = w.s.u32_at(h + 16)? as usize;
    let mut aliases = Vec::new();
    if let Some(array) = w.inline_array(h + 8, SND_ALIAS_LEN * count, 3)? {
        for i in 0..count {
            let a = array + SND_ALIAS_LEN * i;
            let alias_name = name_field(w, a)?;
            let subtitle = name_field(w, a + 8)?;
            let secondary = name_field(w, a + 16)?;
            let chain = name_field(w, a + 24)?;
            let mixer_group = name_field(w, a + 32)?;
            let sound = sound_file(w, a + 40)?;
            // Curves and the Doppler preset are sub-assets: references to
            // them name the field that loaded them.
            w.asset(AssetType::SoundCurve, a + 120)?;
            w.asset(AssetType::LpfCurve, a + 128)?;
            w.asset(AssetType::ReverbCurve, a + 136)?;
            named_struct(w, a + 160, SPEAKER_MAP_LEN, 8, false)?;
            named_struct(w, a + 176, OCCLUSION_SHAPE_LEN, 8, false)?;
            w.asset(AssetType::DopplerPreset, a + 192)?;
            aliases.push(SoundAlias {
                name: alias_name,
                subtitle,
                secondary,
                chain,
                mixer_group,
                sound,
                sequence: w.s.i32_at(a + 48)?,
                vol_min: w.s.f32_at(a + 52)?,
                vol_max: w.s.f32_at(a + 56)?,
                vol_mod_index: w.s.i32_at(a + 60)?,
                pitch_min: w.s.f32_at(a + 64)?,
                pitch_max: w.s.f32_at(a + 68)?,
                dist_min: w.s.f32_at(a + 72)?,
                dist_max: w.s.f32_at(a + 76)?,
                flags: w.s.u32_at(a + 84)?,
                probability: w.s.f32_at(a + 100)?,
                start_delay: w.s.i32_at(a + 112)?,
            });
        }
    }
    w.s.pop();
    w.cap.sounds.push(SoundAliasList { offset: h, end: w.s.pos(), name, aliases });
    Ok(AssetId { ty: AssetType::Sound, index: (w.cap.sounds.len() - 1) as u32 })
}
