//! Modern Warfare III's sounds by alias, read from the owner's install: the
//! sndassets and alias links of some zones ([`crate::sound`]), the KAPI
//! packages holding their streams, and the Opus decode to 16-bit PCM
//! (`opus-decoder`, pure Rust, MIT/Apache-2.0).
//!
//! A stream's package row is found by its key in any `.xsub` of the zone
//! folder; the same key is in every language's pack (`eng_`, `ens_`), so the
//! preferred language's pack is asked first.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::oodle::Decompressor;
use crate::sound::{self, AliasLink, FORMAT_OPUS, SndAsset};
use crate::xsub::{self, XSubEntry, XSubError};

#[derive(Debug)]
pub enum SoundError {
    Io(std::io::Error),
    Package(XSubError),
    /// No package of the zone folder has the stream's key.
    NoStream(u64),
    Format(u8),
    Opus(String),
}

impl std::fmt::Display for SoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Package(e) => write!(f, "{e}"),
            Self::NoStream(k) => write!(f, "no package holds stream {k:016x}"),
            Self::Format(x) => write!(f, "sound format {x} (only Opus, 5, is read)"),
            Self::Opus(e) => write!(f, "opus: {e}"),
        }
    }
}

impl std::error::Error for SoundError {}

impl From<std::io::Error> for SoundError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<XSubError> for SoundError {
    fn from(e: XSubError) -> Self {
        Self::Package(e)
    }
}

/// One sound: its record, and its preload's bytes when the whole stream is
/// in the zone.
#[derive(Debug, Clone)]
pub struct Sound {
    pub zone: usize,
    pub asset: SndAsset,
    pub inline: Option<Vec<u8>>,
}

/// Decoded audio, interleaved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcm {
    pub rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>,
}

impl Pcm {
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels.max(1))
    }

    pub fn seconds(&self) -> f32 {
        self.frames() as f32 / self.rate.max(1) as f32
    }

    /// RIFF/WAVE (16-bit PCM) bytes.
    pub fn wav(&self) -> Vec<u8> {
        let data = (self.samples.len() * 2) as u32;
        let block = self.channels * 2;
        let mut w = Vec::with_capacity(44 + data as usize);
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&(36 + data).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&self.channels.to_le_bytes());
        w.extend_from_slice(&self.rate.to_le_bytes());
        w.extend_from_slice(&(self.rate * u32::from(block)).to_le_bytes());
        w.extend_from_slice(&block.to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&data.to_le_bytes());
        for s in &self.samples {
            w.extend_from_slice(&s.to_le_bytes());
        }
        w
    }
}

/// Decode a sound's Opus packets. `frames` trims the decoder's tail.
pub fn decode_opus<'a>(
    packets: impl Iterator<Item = &'a [u8]>,
    rate: u32,
    channels: u8,
    frames: u32,
) -> Result<Pcm, SoundError> {
    let ch = usize::from(channels);
    if ch == 0 || ch > 2 {
        return Err(SoundError::Opus(format!("{ch} channels")));
    }
    // Opus decodes at 48 kHz (or 8/12/16/24); others are played at 48 kHz.
    let out_rate = if matches!(rate, 8000 | 12000 | 16000 | 24000 | 48000) {
        rate
    } else {
        48000
    };
    let mut dec = opus_decoder::OpusDecoder::new(out_rate, ch)
        .map_err(|e| SoundError::Opus(e.to_string()))?;
    let mut buf = vec![0i16; dec.max_frame_size_per_channel() * ch];
    let mut samples = Vec::with_capacity(frames as usize * ch);
    for p in packets {
        let n = dec
            .decode(p, &mut buf, false)
            .map_err(|e| SoundError::Opus(e.to_string()))?;
        samples.extend_from_slice(&buf[..n * ch]);
    }
    let want = frames as usize * ch;
    if want > 0 && samples.len() > want {
        samples.truncate(want);
    }
    Ok(Pcm {
        rate: out_rate,
        channels: u16::from(channels),
        samples,
    })
}

/// Where a stream key's bytes are.
#[derive(Debug, Clone, Copy)]
struct Row {
    package: usize,
    entry: XSubEntry,
}

/// The sounds of some zones, by alias hash.
#[derive(Default)]
pub struct SoundBank {
    pub zones: Vec<String>,
    pub sounds: Vec<Sound>,
    /// Alias hash -> its sounds (several: the alias's variants).
    pub aliases: HashMap<u64, Vec<usize>>,
    /// Sndasset name hash -> sound.
    pub by_name: HashMap<u64, usize>,
    packages: Vec<PathBuf>,
    rows: HashMap<u64, Row>,
}

impl SoundBank {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a decompressed zone's sounds and aliases. Returns (sounds, links).
    pub fn add_zone(&mut self, name: &str, zone: &[u8]) -> (usize, usize) {
        let Some(list) = sound::asset_list(zone) else {
            return (0, 0);
        };
        let want = list
            .types
            .iter()
            .filter(|&&t| t == sound::ASSET_SNDASSET)
            .count();
        if want == 0 {
            return (0, 0);
        }
        let found = sound::find_sndassets(zone, list.data_start);
        if found.len() != want {
            // Numbering would be off: keep none rather than wrong links.
            return (0, 0);
        }
        let zi = self.zones.len();
        self.zones.push(name.to_owned());
        let base = self.sounds.len();
        for asset in found {
            let inline = asset
                .fully_preloaded()
                .then(|| zone[asset.preload.clone().unwrap()].to_vec());
            self.by_name.entry(asset.name).or_insert(self.sounds.len());
            self.sounds.push(Sound {
                zone: zi,
                asset,
                inline,
            });
        }
        let links = sound::alias_links(zone, &list);
        for AliasLink {
            alias, sndasset, ..
        } in &links
        {
            let v = self.aliases.entry(*alias).or_default();
            let s = base + sndasset;
            if !v.contains(&s) {
                v.push(s);
            }
        }
        (want, links.len())
    }

    /// Find the stream rows of every sound added so far in the `.xsub`
    /// packages of `zone_dir`, the `lang` ones (e.g. `eng`) first.
    pub fn index_packages(&mut self, zone_dir: &Path, lang: &str) -> Result<usize, SoundError> {
        let mut want: Vec<u64> = self
            .sounds
            .iter()
            .filter(|s| s.inline.is_none() && !self.rows.contains_key(&s.asset.key))
            .map(|s| s.asset.key)
            .collect();
        want.sort_unstable();
        want.dedup();
        let mut files: Vec<PathBuf> = std::fs::read_dir(zone_dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "xsub"))
            .collect();
        let lang_prefix = format!("{lang}_");
        let rank = |p: &PathBuf| {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if n.starts_with(&lang_prefix) {
                0
            } else if n.len() > 4
                && n.as_bytes()[3] == b'_'
                && n[..3].chars().all(|c| c.is_ascii_lowercase())
            {
                // another language's pack (`ens_`, ...): last
                2
            } else {
                1
            }
        };
        files.sort_by_key(|p| (rank(p), p.clone()));
        let mut found = 0usize;
        for p in files {
            if want.is_empty() {
                break;
            }
            let Ok((_, entries)) = xsub::read_index(&mut std::fs::File::open(&p)?) else {
                continue;
            };
            let pi = self.packages.len();
            let mut used = false;
            want.retain(|k| match entries.binary_search_by_key(k, |e| e.key) {
                Ok(i) => {
                    self.rows.insert(
                        *k,
                        Row {
                            package: pi,
                            entry: entries[i],
                        },
                    );
                    used = true;
                    found += 1;
                    false
                }
                Err(_) => true,
            });
            if used {
                self.packages.push(p);
            }
        }
        Ok(found)
    }

    /// The package a sound streams from (None: inline, or not found).
    pub fn package_of(&self, sound: &Sound) -> Option<&Path> {
        self.row_of(sound).map(|(p, _)| p)
    }

    /// The package and its row holding a sound's stream.
    pub fn row_of(&self, sound: &Sound) -> Option<(&Path, XSubEntry)> {
        self.rows
            .get(&sound.asset.key)
            .map(|r| (self.packages[r.package].as_path(), r.entry))
    }

    /// The sound's stream bytes (seek table + packets), or its inline
    /// preload (packets only; the table length is then 0).
    pub fn stream(
        &self,
        sound: &Sound,
        oodle: &dyn Decompressor,
    ) -> Result<(Vec<u8>, usize), SoundError> {
        if let Some(b) = &sound.inline {
            return Ok((b.clone(), 0));
        }
        let row = self
            .rows
            .get(&sound.asset.key)
            .ok_or(SoundError::NoStream(sound.asset.key))?;
        let mut f = std::fs::File::open(&self.packages[row.package])?;
        use std::io::{Read, Seek, SeekFrom};
        let mut blob = vec![0u8; row.entry.size as usize];
        f.seek(SeekFrom::Start(row.entry.offset))?;
        f.read_exact(&mut blob)?;
        let bytes = xsub::decode_entry(&row.entry, &blob, oodle)?;
        Ok((bytes, sound.asset.table_len as usize))
    }

    /// Decode a sound to PCM.
    pub fn decode(&self, sound: &Sound, oodle: &dyn Decompressor) -> Result<Pcm, SoundError> {
        if sound.asset.format != FORMAT_OPUS {
            return Err(SoundError::Format(sound.asset.format));
        }
        let (bytes, table) = self.stream(sound, oodle)?;
        decode_opus(
            sound::packets(&bytes, table),
            sound.asset.rate,
            sound.asset.channels,
            sound.asset.frames,
        )
    }

    /// The sounds of an alias.
    pub fn alias(&self, name: &str) -> Option<&[usize]> {
        self.alias_by_hash(sound::alias_hash(name))
    }

    pub fn alias_by_hash(&self, hash: u64) -> Option<&[usize]> {
        self.aliases.get(&hash).map(Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header() {
        let p = Pcm {
            rate: 48000,
            channels: 1,
            samples: vec![1, -1, 2],
        };
        let w = p.wav();
        assert_eq!(&w[..4], b"RIFF");
        assert_eq!(w.len(), 44 + 6);
        assert_eq!(u32::from_le_bytes(w[40..44].try_into().unwrap()), 6);
        assert_eq!(p.frames(), 3);
    }

    #[test]
    fn decodes_opus_silence() {
        // 0xF8 0xFF 0xFE: one 20 ms CELT fullband frame of silence.
        let pkts = [[0xf8u8, 0xff, 0xfe]; 3];
        let pcm = decode_opus(pkts.iter().map(|p| &p[..]), 48000, 1, 2000).unwrap();
        assert_eq!(pcm.frames(), 2000);
        assert!(pcm.samples.iter().all(|&s| s.unsigned_abs() < 4));
    }
}
