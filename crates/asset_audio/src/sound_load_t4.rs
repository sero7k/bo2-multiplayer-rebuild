//! bo2mp: Call of Duty: World at War's sounds for its guest guns in a Black
//! Ops II match (`waw_springfield_mp`, `waw_mp40_mp`).
//!
//! The `t4` reader loads World at War's `common_mp` (every asset, read to
//! the end); its sound alias lists (`snd_alias_list_t`) go into the catalog
//! under the World at War namespace (`AssetNamespace::T4`, which the guns'
//! weapon rows carry), so they never take the name of another game's alias.
//!
//! A World at War loaded sound is a whole RIFF file in the zone: a WAVE file
//! of Microsoft ADPCM (format 2) for nearly all of them, some of 16-bit PCM
//! (format 1), and a few xWMA files (WMA2, format 0x161, with their packet
//! table; those in the profile Black Ops' WMA2 reader knows are read). The
//! file is kept as it is (`T4_WAV`) and decoded when it is first played
//! (`decode_t4_wav`). Each file's rate is its own (the variants of one sound
//! are pitched by their headers' rates). Streamed sounds are named by
//! directory and file under the install's IWDs (`sound/<dir>/<name>`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use t4::zone::{AssetType, GNode, Zone};

use crate::sound_catalog::{CapturedAlias, CapturedSound, LoadedSoundPcm, SoundCatalog};
use crate::{AssetNamespace, GamesRoot, ZoneGame, ZoneOwner};

/// `LoadedSoundPcm::format` of a World at War loaded sound: its bytes are a
/// whole RIFF WAVE or xWMA file (see `decode_t4_wav`).
pub const T4_WAV: i32 = 0x7434;

const WAVE_PCM: u16 = 1;
const WAVE_MS_ADPCM: u16 = 2;
const WAVE_WMA2: u16 = 0x161;

/// `SoundFile::type`.
const SAT_LOADED: u8 = 1;
const SAT_STREAMED: u8 = 2;

/// World at War's multiplayer common zone, when it is installed.
pub fn waw_common_mp(games: &GamesRoot) -> Option<PathBuf> {
    asset_transport::find_common_mp_for_envelope(games, asset_transport::T4_ZONE_VERSION)
        .ok()
        .map(|zone| zone.path)
}

/// The install's `main` (its IWDs, where streamed sounds are), beside its
/// common zone.
pub fn waw_main(games: &GamesRoot) -> Option<PathBuf> {
    let common = waw_common_mp(games)?;
    let main = asset_transport::game_root_for_zone(&common)
        .ok()?
        .join("main");
    main.is_dir().then_some(main)
}

fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<SoundCatalog>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<SoundCatalog>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The cache key: the zone's real path (the guns' lane and the sound bank
/// may find it through different roots).
fn cache_key(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn cached(path: &Path) -> Option<Arc<SoundCatalog>> {
    cache()
        .lock()
        .ok()
        .and_then(|c| c.get(&cache_key(path)).cloned())
}

/// The sounds of World at War's `common_mp` at `path` (read once per
/// process; `waw_sounds_from_zone` fills the same cache from a read someone
/// else made).
pub fn waw_weapon_sounds(path: &Path) -> Result<Arc<SoundCatalog>, String> {
    // One read at a time: a second caller takes the first one's catalog.
    static READING: Mutex<()> = Mutex::new(());
    let _one = READING.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(hit) = cached(path) {
        return Ok(hit);
    }
    let data = t4::fastfile::load(path).map_err(|e| format!("{e:#}"))?;
    let zone =
        Zone::parse(&data, t4::zone::ParseOptions::default()).map_err(|e| format!("{e:#}"))?;
    drop(data);
    Ok(waw_sounds_from_zone(path, &zone))
}

/// The sounds of an already-read World at War `common_mp` (`path` is where
/// it was read from), kept for `waw_weapon_sounds`.
pub fn waw_sounds_from_zone(path: &Path, zone: &Zone) -> Arc<SoundCatalog> {
    if let Some(hit) = cached(path) {
        return hit;
    }
    let catalog = Arc::new(load_sound_catalog_t4(path, zone));
    if let Ok(mut c) = cache().lock() {
        c.entry(cache_key(path)).or_insert_with(|| catalog.clone());
    }
    catalog
}

/// One World at War zone's sound alias lists (see the module docs).
pub fn load_sound_catalog_t4(path: &Path, zone: &Zone) -> SoundCatalog {
    let owner = ZoneOwner::from_zone_path(path);
    let mut catalog = SoundCatalog::default();
    catalog.set_capture_game(ZoneGame::T6);
    catalog.set_capture_zone(owner);
    if let Some(stop) = &zone.stats.stopped_at {
        diag::warn!(
            Zone,
            "sound bank: World at War {} read {} of {} assets (stopped at {stop})",
            path.display(),
            zone.stats.top_level_parsed,
            zone.stats.top_level_total
        );
    }
    let mut seen: HashMap<String, bool> = HashMap::new();
    let (mut lists, mut clips, mut bytes, mut undecodable) = (0usize, 0usize, 0usize, 0usize);
    for (_, asset) in zone.of_type(AssetType::Sound) {
        let Some(name) = asset.root.string("aliasName").filter(|n| !n.is_empty()) else {
            continue;
        };
        let mut aliases = Vec::new();
        for row in asset.root.nodes("head") {
            let mut alias = alias_row(name, row);
            let file = row.node("soundFile");
            let kind = file.and_then(|f| f.data.first().copied());
            let reference = file.and_then(|f| f.node("u"));
            match (kind, reference) {
                (Some(SAT_LOADED), Some(u)) => {
                    let loaded = u.asset("loadSnd").and_then(|id| zone.assets.get(id));
                    let lname = loaded
                        .and_then(|l| l.root.string("name"))
                        .unwrap_or("")
                        .to_owned();
                    let data = loaded
                        .and_then(|l| l.root.node("sound"))
                        .map_or(&[][..], |s| s.bytes("data"));
                    alias.file_type = Some(1);
                    if lname.is_empty() || data.is_empty() {
                        alias.file_name = Some("null.wav".to_owned());
                    } else {
                        let usable =
                            *seen.entry(lname.clone()).or_insert_with(|| {
                                match loaded_sound(&lname, data) {
                                    Some(mut pcm) => {
                                        pcm.zone = owner;
                                        clips += 1;
                                        bytes += data.len();
                                        catalog.ingest_loaded(pcm);
                                        true
                                    }
                                    None => {
                                        undecodable += 1;
                                        false
                                    }
                                }
                            });
                        alias.file_exists = Some(u8::from(usable));
                        alias.file_name = Some(lname.clone());
                        // (A clip no decoder reads binds nothing: the alias
                        // is reported missing, never given another sound.)
                        alias.loaded_name = Some(lname);
                    }
                }
                (Some(SAT_STREAMED), Some(u)) => {
                    let file = u.node("streamSnd").and_then(|s| s.node("filename"));
                    let dir = file
                        .and_then(|f| f.string("dir"))
                        .unwrap_or("")
                        .replace('\\', "/");
                    let leaf = file.and_then(|f| f.string("name")).unwrap_or("").to_owned();
                    alias.file_type = Some(2);
                    alias.file_name = Some(if dir.is_empty() {
                        leaf.clone()
                    } else {
                        format!("{dir}/{leaf}")
                    });
                    alias.file_exists = Some(u8::from(!leaf.is_empty()));
                    alias.streamed = (!leaf.is_empty()).then_some((dir, leaf));
                }
                _ => {
                    alias.file_type = Some(1);
                    alias.file_name = Some("null.wav".to_owned());
                }
            }
            aliases.push(alias);
        }
        lists += 1;
        catalog.ingest_sound(CapturedSound {
            name: name.to_owned(),
            aliases,
            game: ZoneGame::T6,
            zone: owner,
            namespace: Some(AssetNamespace::T4),
        });
    }
    catalog.finalize();
    diag::info!(
        Zone,
        "sound bank: World at War {}: {lists} alias lists, {clips} loaded clips ({:.1} MB), {undecodable} in a format not read",
        path.display(),
        bytes as f64 / 1.0e6
    );
    catalog
}

fn alias_row(list_name: &str, row: &GNode) -> CapturedAlias {
    let text = |field: &str| {
        row.string(field)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    let pitch = |v: f32| if v.is_finite() && v > 0.0 { v } else { 1.0 };
    let probability = row.float("probability");
    CapturedAlias {
        alias_name: list_name.to_owned(),
        subtitle: text("subtitle"),
        secondary: text("secondaryAliasName"),
        chain: text("chainAliasName"),
        sequence: row.int("sequence") as i32,
        vol_min: row.float("volMin").max(0.0),
        vol_max: row.float("volMax").max(0.0),
        pitch_min: pitch(row.float("pitchMin")),
        pitch_max: pitch(row.float("pitchMax")),
        dist_min: row.float("distMin").max(0.0),
        dist_max: row.float("distMax").max(0.0),
        // (World at War's flag word is not IW4's: left unread, so nothing is
        // taken for a loop or a channel it is not.)
        flags: None,
        slave_percentage: row.float("slavePercentage"),
        probability: if probability > 0.0 { probability } else { 1.0 },
        lfe_percentage: row.float("lfePercentage"),
        center_percentage: row.float("centerPercentage"),
        start_delay: row.int("startDelay") as i32,
        envelop_min: row.float("envelopMin"),
        envelop_max: row.float("envelopMax"),
        envelop_percentage: row.float("envelopPercentage"),
        ..CapturedAlias::default()
    }
}

/// A loaded sound's catalog entry (its WAVE file kept whole), if a decoder
/// here reads its format.
fn loaded_sound(name: &str, data: &[u8]) -> Option<LoadedSoundPcm> {
    let wav = parse_wav(data).ok()?;
    if !wav.decodable() {
        return None;
    }
    let mut pcm = LoadedSoundPcm::captured(
        name,
        T4_WAV,
        wav.rate,
        i32::from(wav.channels),
        data.to_vec(),
        Vec::new(),
    );
    pcm.samples = wav.frames();
    pcm.block_size = u32::from(wav.block_align);
    pcm.namespace = Some(AssetNamespace::T4);
    Some(pcm)
}

/// A WAVE file's format and where its samples are.
#[derive(Clone, Debug)]
pub struct WavInfo {
    pub tag: u16,
    pub channels: u16,
    pub rate: u32,
    pub block_align: u16,
    pub bits: u16,
    /// The ADPCM coefficient pairs (Microsoft ADPCM only).
    pub coefs: Vec<(i32, i32)>,
    pub data: std::ops::Range<usize>,
    /// The `fact` chunk's frame count, if it has one.
    pub fact_frames: Option<u32>,
    /// An xWMA file's packet table (`dpds`: decoded bytes after each
    /// packet).
    pub packets: Vec<u32>,
}

impl WavInfo {
    fn decodable(&self) -> bool {
        match self.tag {
            WAVE_PCM => matches!(self.bits, 8 | 16),
            WAVE_MS_ADPCM => self.block_align as usize > 7 * self.channels as usize,
            WAVE_WMA2 => {
                !self.packets.is_empty()
                    && crate::T5WmaProfile::from_geometry(u32::from(self.channels), self.rate)
                        .is_some_and(|p| p.packet_bytes() == usize::from(self.block_align))
            }
            _ => false,
        }
    }

    /// Its length in frames (0 when only decoding tells).
    pub fn frames(&self) -> u32 {
        let len = self.data.len();
        let ch = usize::from(self.channels.max(1));
        let n = match self.tag {
            WAVE_PCM if self.bits >= 8 => len / (ch * usize::from(self.bits / 8)),
            WAVE_WMA2 => {
                let decoded = self.packets.last().copied().unwrap_or(0) as usize;
                decoded / (2 * ch)
            }
            WAVE_MS_ADPCM => {
                let align = usize::from(self.block_align);
                let per_block = ms_adpcm_block_frames(align, ch);
                let full = len / align * per_block;
                let rest = len % align;
                full + if rest >= 7 * ch {
                    ms_adpcm_block_frames(rest, ch)
                } else {
                    0
                }
            }
            _ => 0,
        };
        let n = u32::try_from(n).unwrap_or(u32::MAX);
        self.fact_frames
            .map_or(n, |f| if n == 0 { f } else { n.min(f) })
    }
}

fn ms_adpcm_block_frames(block: usize, channels: usize) -> usize {
    2 + (block - 7 * channels) * 2 / channels
}

/// The standard Microsoft ADPCM coefficients.
const MS_ADPCM_COEFS: [(i32, i32); 7] = [
    (256, 0),
    (512, -256),
    (0, 0),
    (192, 64),
    (240, 0),
    (460, -208),
    (392, -232),
];

const MS_ADPCM_ADAPT: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

/// Read a RIFF WAVE (or xWMA) file's `fmt ` (and `fact`, `dpds`) and find
/// its `data`.
pub fn parse_wav(bytes: &[u8]) -> Result<WavInfo, String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || !matches!(&bytes[8..12], b"WAVE" | b"XWMA") {
        return Err("not a RIFF WAVE or xWMA file".to_owned());
    }
    let u16_at = |at: usize| {
        bytes
            .get(at..at + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    let u32_at = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let mut at = 12;
    let mut fmt = None;
    let mut data = None;
    let mut fact_frames = None;
    let mut packets = Vec::new();
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let len = u32_at(at + 4).unwrap_or(0) as usize;
        let body = at + 8;
        let end = body.saturating_add(len).min(bytes.len());
        match id {
            b"fmt " if len >= 16 => {
                let tag = u16_at(body).unwrap_or(0);
                let channels = u16_at(body + 2).unwrap_or(0);
                let rate = u32_at(body + 4).unwrap_or(0);
                let block_align = u16_at(body + 12).unwrap_or(0);
                let bits = u16_at(body + 14).unwrap_or(0);
                let mut coefs = Vec::new();
                if tag == WAVE_MS_ADPCM && len >= 22 {
                    let count = usize::from(u16_at(body + 20).unwrap_or(0));
                    for k in 0..count {
                        let p = body + 22 + k * 4;
                        if p + 4 > end {
                            break;
                        }
                        let c1 = i16::from_le_bytes([bytes[p], bytes[p + 1]]);
                        let c2 = i16::from_le_bytes([bytes[p + 2], bytes[p + 3]]);
                        coefs.push((i32::from(c1), i32::from(c2)));
                    }
                }
                fmt = Some((tag, channels, rate, block_align, bits, coefs));
            }
            b"fact" if len >= 4 => fact_frames = u32_at(body),
            b"dpds" => {
                packets = (body..end.saturating_sub(3))
                    .step_by(4)
                    .filter_map(u32_at)
                    .collect()
            }
            b"data" => data = Some(body..end),
            _ => {}
        }
        at = body.saturating_add(len).saturating_add(len & 1);
    }
    let (tag, channels, rate, block_align, bits, coefs) = fmt.ok_or("no `fmt ` chunk")?;
    let data = data.ok_or("no `data` chunk")?;
    if channels == 0 || rate == 0 {
        return Err(format!("bad format: {channels} channels at {rate} Hz"));
    }
    Ok(WavInfo {
        tag,
        channels,
        rate,
        block_align,
        bits,
        coefs,
        data,
        fact_frames,
        packets,
    })
}

/// A decoded World at War clip: interleaved signed 16-bit samples.
#[derive(Clone, Debug, Default)]
pub struct T4Pcm {
    pub channels: u16,
    pub rate: u32,
    pub samples: Vec<i16>,
}

impl T4Pcm {
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels.max(1))
    }
}

/// Decode a World at War loaded sound (a RIFF file: 8/16-bit PCM or
/// Microsoft ADPCM WAVE, or xWMA in the 44.1 kHz mono profile Black Ops'
/// WMA2 reader knows).
pub fn decode_t4_wav(bytes: &[u8]) -> Result<T4Pcm, String> {
    let wav = parse_wav(bytes)?;
    let data = &bytes[wav.data.clone()];
    let ch = usize::from(wav.channels);
    let mut samples = match wav.tag {
        WAVE_PCM if wav.bits == 16 => data
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect(),
        WAVE_PCM if wav.bits == 8 => data.iter().map(|&b| (i16::from(b) - 128) << 8).collect(),
        WAVE_MS_ADPCM => {
            let coefs = if wav.coefs.is_empty() {
                MS_ADPCM_COEFS.to_vec()
            } else {
                wav.coefs.clone()
            };
            decode_ms_adpcm(data, ch, usize::from(wav.block_align), &coefs)?
        }
        WAVE_WMA2 => decode_wma2(data, &wav)?,
        tag => {
            return Err(format!(
                "WAVE format {tag:#x} ({} bits) is not read",
                wav.bits
            ));
        }
    };
    let mut frames = samples.len() / ch;
    if let Some(f) = wav.fact_frames.filter(|&f| f > 0) {
        frames = frames.min(f as usize);
    }
    samples.truncate(frames * ch);
    if samples.is_empty() {
        return Err("no samples".to_owned());
    }
    Ok(T4Pcm {
        channels: wav.channels,
        rate: wav.rate,
        samples,
    })
}

/// Microsoft ADPCM: per block, each channel's predictor, delta and two
/// starting samples, then 4-bit codes (high nibble first, channels
/// interleaved).
fn decode_ms_adpcm(
    data: &[u8],
    ch: usize,
    align: usize,
    coefs: &[(i32, i32)],
) -> Result<Vec<i16>, String> {
    if ch == 0 || ch > 2 || align <= 7 * ch {
        return Err(format!(
            "bad MS ADPCM layout: {ch} channels, {align}-byte blocks"
        ));
    }
    let mut out =
        Vec::with_capacity(data.len() / align * ms_adpcm_block_frames(align, ch) * ch + 2 * ch);
    for block in data.chunks(align) {
        if block.len() < 7 * ch {
            break;
        }
        let i16_at = |at: usize| i32::from(i16::from_le_bytes([block[at], block[at + 1]]));
        let mut coef = [(0i32, 0i32); 2];
        let mut delta = [0i32; 2];
        let mut s1 = [0i32; 2];
        let mut s2 = [0i32; 2];
        for c in 0..ch {
            let pred = usize::from(block[c]);
            coef[c] = *coefs
                .get(pred)
                .ok_or_else(|| format!("MS ADPCM predictor {pred} out of range"))?;
            delta[c] = i16_at(ch + 2 * c);
            s1[c] = i16_at(3 * ch + 2 * c);
            s2[c] = i16_at(5 * ch + 2 * c);
        }
        for c in 0..ch {
            out.push(s2[c] as i16);
        }
        for c in 0..ch {
            out.push(s1[c] as i16);
        }
        let mut c = 0usize;
        for &byte in &block[7 * ch..] {
            for nibble in [byte >> 4, byte & 0x0f] {
                let code = i32::from(nibble);
                let signed = if code >= 8 { code - 16 } else { code };
                // (A division, truncating, as the reference decoders do.)
                let predicted = (s1[c] * coef[c].0 + s2[c] * coef[c].1) / 256 + signed * delta[c];
                let sample = predicted.clamp(i32::from(i16::MIN), i32::from(i16::MAX));
                s2[c] = s1[c];
                s1[c] = sample;
                delta[c] = ((MS_ADPCM_ADAPT[nibble as usize] * delta[c]) >> 8).max(16);
                out.push(sample as i16);
                c = (c + 1) % ch;
            }
        }
    }
    Ok(out)
}

#[derive(Default)]
struct WmaSamples(Vec<i16>);

impl crate::XwmaPcmSink for WmaSamples {
    type Workspace = ();

    fn reserve_workspace(&mut self, _bytes: usize) -> Result<(), crate::XwmaDecodeError> {
        Ok(())
    }

    fn extend(&mut self, samples: &[i16]) -> Result<(), crate::XwmaDecodeError> {
        self.0.extend_from_slice(samples);
        Ok(())
    }

    fn sample_count(&self) -> usize {
        self.0.len()
    }

    fn visit_samples(
        &self,
        visitor: &mut dyn FnMut(&[i16]) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        visitor(&self.0)
    }
}

/// xWMA packets, through Black Ops' WMA2 reader (the profiles it knows),
/// cut to the length the packet table says.
fn decode_wma2(data: &[u8], wav: &WavInfo) -> Result<Vec<i16>, String> {
    if wav.packets.is_empty() {
        return Err("xWMA without a packet table".to_owned());
    }
    let mut out = WmaSamples::default();
    crate::wma_t5::decode(
        data,
        &wav.packets,
        u32::from(wav.channels),
        wav.rate,
        &mut out,
    )
    .map_err(|e| e.to_string())?;
    let decoded = wav.packets.last().copied().unwrap_or(0) as usize / 2;
    if decoded > 0 && out.0.len() > decoded {
        out.0.truncate(decoded);
    }
    Ok(out.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(
        tag: u16,
        channels: u16,
        rate: u32,
        align: u16,
        bits: u16,
        extra: &[u8],
        data: &[u8],
    ) -> Vec<u8> {
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&tag.to_le_bytes());
        fmt.extend_from_slice(&channels.to_le_bytes());
        fmt.extend_from_slice(&rate.to_le_bytes());
        fmt.extend_from_slice(&(rate * u32::from(align)).to_le_bytes());
        fmt.extend_from_slice(&align.to_le_bytes());
        fmt.extend_from_slice(&bits.to_le_bytes());
        if !extra.is_empty() {
            fmt.extend_from_slice(&(extra.len() as u16).to_le_bytes());
            fmt.extend_from_slice(extra);
        }
        let mut out = b"RIFF\0\0\0\0WAVE".to_vec();
        for (id, body) in [
            (b"fmt ", &fmt[..]),
            (b"PRIV", &[1u8, 2, 3][..]),
            (b"data", data),
        ] {
            out.extend_from_slice(id);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(body);
            if body.len() % 2 == 1 {
                out.push(0);
            }
        }
        let len = (out.len() - 8) as u32;
        out[4..8].copy_from_slice(&len.to_le_bytes());
        out
    }

    fn ms_extra(spb: u16) -> Vec<u8> {
        let mut e = spb.to_le_bytes().to_vec();
        e.extend_from_slice(&7u16.to_le_bytes());
        for (a, b) in MS_ADPCM_COEFS {
            e.extend_from_slice(&(a as i16).to_le_bytes());
            e.extend_from_slice(&(b as i16).to_le_bytes());
        }
        e
    }

    #[test]
    fn pcm16_reads_through_odd_chunks() {
        let data: Vec<u8> = [100i16, -200, 300]
            .iter()
            .flat_map(|s| s.to_le_bytes())
            .collect();
        let pcm = decode_t4_wav(&wav(1, 1, 44301, 2, 16, &[], &data)).unwrap();
        assert_eq!((pcm.channels, pcm.rate), (1, 44301));
        assert_eq!(pcm.samples, vec![100, -200, 300]);
    }

    #[test]
    fn ms_adpcm_mono_block() {
        // Predictor 0 (256, 0), delta 16, sample1 1000, sample2 500; codes
        // +1 then -1: 1000 + 16 = 1016 (delta 16*230/256 -> 16), then
        // 1016 - 16 = 1000.
        let mut block = vec![0u8];
        block.extend_from_slice(&16i16.to_le_bytes());
        block.extend_from_slice(&1000i16.to_le_bytes());
        block.extend_from_slice(&500i16.to_le_bytes());
        block.push(0x1f);
        let file = wav(2, 1, 22050, 8, 4, &ms_extra(4), &block);
        let info = parse_wav(&file).unwrap();
        assert_eq!(info.frames(), 4);
        assert!(info.decodable());
        let pcm = decode_t4_wav(&file).unwrap();
        assert_eq!(pcm.samples, vec![500, 1000, 1016, 1000]);
    }

    #[test]
    fn ms_adpcm_stereo_interleaves_nibbles() {
        let mut block = vec![0u8, 0];
        for v in [16i16, 16, 0, 0, 0, 0] {
            block.extend_from_slice(&v.to_le_bytes());
        }
        block.push(0x21);
        let pcm = decode_t4_wav(&wav(2, 2, 44100, 15, 4, &ms_extra(4), &block)).unwrap();
        assert_eq!(pcm.channels, 2);
        assert_eq!(pcm.samples, vec![0, 0, 0, 0, 32, 16]);
    }

    #[test]
    fn unknown_formats_are_errors_not_noise() {
        assert!(decode_t4_wav(&wav(0x55, 1, 44100, 1, 0, &[], &[0; 16])).is_err());
        assert!(decode_t4_wav(b"RIFX").is_err());
        assert!(
            !parse_wav(&wav(0x161, 1, 32000, 1280, 16, &[], &[0; 16]))
                .unwrap()
                .decodable()
        );
    }
}
