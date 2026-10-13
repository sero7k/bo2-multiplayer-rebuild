//! bo2mp: a Ghosts interface sound held in a zone (a loaded, 16-bit PCM
//! `LoadedSound`), read out as samples: the match start countdown's tick
//! (`match_countdown_tick` in `code_post_gfx_mp`) plays from it.

use std::path::Path;

use crate::sound::{LoadedRef, SoundData};
use crate::zone::Zone;

/// The zone and alias of Ghosts' match start countdown tick.
pub const COUNTDOWN_ZONE: &str = "code_post_gfx_mp";
pub const COUNTDOWN_ALIAS: &str = "match_countdown_tick";

/// A loaded sound's samples (interleaved, -1..1).
#[derive(Clone, Debug)]
pub struct UiPcm {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

/// The first variant of `alias` in `<install>/zone/<zone>.ff`, when it is
/// a loaded 16-bit PCM sound.
pub fn loaded_pcm(install: &Path, zone: &str, alias: &str) -> Result<UiPcm, String> {
    let path = install.join("zone").join(format!("{zone}.ff"));
    let zone = Zone::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let scan = zone.sounds();
    let list = scan.find(alias).ok_or_else(|| format!("no alias `{alias}`"))?;
    let loaded = list
        .aliases
        .iter()
        .find_map(|a| match a.sound.get().map(|f| &f.data) {
            Some(SoundData::Loaded(LoadedRef::Inline(l))) => Some(l.clone()),
            _ => None,
        })
        .ok_or_else(|| format!("`{alias}` is not a loaded sound in this zone"))?;
    if loaded.format != 1 || loaded.bits != 16 || loaded.channels == 0 {
        return Err(format!("`{alias}`: format {} {} bit, not 16-bit PCM", loaded.format, loaded.bits));
    }
    let at = loaded.data_offset.ok_or_else(|| format!("`{alias}`: its samples are not in the zone"))?;
    let len = loaded.data_len as usize;
    let bytes = zone
        .image
        .get(at..at + len)
        .ok_or_else(|| format!("`{alias}`: samples past the zone ({at}+{len})"))?;
    let samples = bytes
        .chunks_exact(2)
        .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0)
        .collect();
    Ok(UiPcm { samples, channels: loaded.channels, sample_rate: loaded.sample_rate })
}
