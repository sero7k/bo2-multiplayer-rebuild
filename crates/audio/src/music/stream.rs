//! A music file played as it decodes: the file's bytes stay in memory (a few
//! megabytes) and symphonia turns them into samples packet by packet on the
//! audio thread, so a four-minute MP3 never becomes 80 MB of `f32`s. MP3,
//! Ogg Vorbis, FLAC and WAV are read (symphonia's `mp3`, `ogg`, `vorbis`,
//! `flac`, `wav` and `pcm` features).

use std::io::Cursor;
use std::num::NonZero;
use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{ChannelCount, Decodable, SampleRate, Source};
use bevy::prelude::*;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// The extensions read from the music folders.
pub const MUSIC_EXTENSIONS: [&str; 4] = ["mp3", "ogg", "wav", "flac"];

#[derive(Asset, TypePath, Clone)]
pub struct StreamedMusic {
    bytes: Arc<[u8]>,
    ext: Option<String>,
    channels: u16,
    rate: u32,
    length: Option<f64>,
}

/// What a probe of a music file found.
#[derive(Clone, Debug, PartialEq)]
pub struct Probe {
    pub codec: String,
    pub channels: u16,
    pub rate: u32,
    /// Seconds, counted from the packets' own durations (exact for every
    /// format read here, with or without a length header).
    pub length: Option<f64>,
}

fn open(
    bytes: Arc<[u8]>,
    ext: Option<&str>,
) -> Result<(Box<dyn FormatReader>, Box<dyn Decoder>, u32), String> {
    let mss = MediaSourceStream::new(Box::new(Cursor::new(ArcBytes(bytes))), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = ext {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("unrecognised container ({e})"))?;
    let format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "no audio track".to_owned())?;
    let track_id = track.id;
    let decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("unsupported codec ({e})"))?;
    Ok((format, decoder, track_id))
}

/// `AsRef<[u8]>` over the shared bytes, for `Cursor`.
struct ArcBytes(Arc<[u8]>);

impl AsRef<[u8]> for ArcBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Reads the whole file's packet headers (no decoding beyond the first
/// packet) to find its codec, layout and exact length.
pub fn probe(bytes: Arc<[u8]>, ext: Option<&str>) -> Result<Probe, String> {
    let (mut format, mut decoder, track_id) = open(bytes, ext)?;
    let params = format
        .tracks()
        .iter()
        .find(|t| t.id == track_id)
        .map(|t| t.codec_params.clone())
        .ok_or_else(|| "no audio track".to_owned())?;
    let codec = symphonia::default::get_codecs()
        .get_codec(params.codec)
        .map_or_else(|| "unknown".to_owned(), |d| d.short_name.to_owned());
    let mut rate = params.sample_rate.unwrap_or(0);
    let mut channels = params.channels.map_or(0, |c| c.count() as u16);
    let mut ticks: u64 = 0;
    let mut decoded_one = false;
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::ResetRequired) => continue,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }
        ticks += packet.dur;
        if !decoded_one && let Ok(buf) = decoder.decode(&packet) {
            decoded_one = true;
            rate = buf.spec().rate;
            channels = buf.spec().channels.count() as u16;
        }
    }
    if !decoded_one {
        return Err("no packet decoded".to_owned());
    }
    if rate == 0 || channels == 0 {
        return Err("no sample rate or channel layout".to_owned());
    }
    let length = params
        .time_base
        .map(|tb| {
            let t = tb.calc_time(ticks);
            t.seconds as f64 + t.frac
        })
        .or_else(|| (ticks > 0).then(|| ticks as f64 / f64::from(rate)))
        .filter(|l| *l > 0.0);
    Ok(Probe {
        codec,
        channels,
        rate,
        length,
    })
}

impl StreamedMusic {
    pub fn new(bytes: Arc<[u8]>, ext: Option<String>, probe: &Probe) -> Self {
        Self {
            bytes,
            ext,
            channels: probe.channels.max(1),
            rate: probe.rate.max(1),
            length: probe.length,
        }
    }
}

pub struct StreamDecoder {
    reader: Option<(Box<dyn FormatReader>, Box<dyn Decoder>, u32)>,
    buf: Vec<f32>,
    pos: usize,
    scratch: Option<SampleBuffer<f32>>,
    channels: u16,
    rate: u32,
    length: Option<f64>,
}

impl StreamDecoder {
    /// Decodes the next packet into `buf`; `false` at the end.
    fn refill(&mut self) -> bool {
        let Some((format, decoder, track_id)) = self.reader.as_mut() else {
            return false;
        };
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::ResetRequired) => continue,
                Err(_) => {
                    self.reader = None;
                    return false;
                }
            };
            if packet.track_id() != *track_id {
                continue;
            }
            let Ok(decoded) = decoder.decode(&packet) else {
                continue;
            };
            let spec = *decoded.spec();
            let frames = decoded.capacity() as u64;
            if self.scratch.as_ref().is_none_or(|s| s.capacity() < decoded.capacity() * spec.channels.count()) {
                self.scratch = Some(SampleBuffer::<f32>::new(frames, spec));
            }
            let Some(scratch) = self.scratch.as_mut() else {
                return false;
            };
            scratch.copy_interleaved_ref(decoded);
            let src_ch = spec.channels.count().max(1);
            let out_ch = usize::from(self.channels);
            self.buf.clear();
            self.pos = 0;
            if src_ch == out_ch {
                self.buf.extend_from_slice(scratch.samples());
            } else {
                // The layout changed mid-file (rare): fit it to the one the
                // sink was opened with.
                for frame in scratch.samples().chunks(src_ch) {
                    for c in 0..out_ch {
                        self.buf.push(frame.get(c).copied().unwrap_or(frame[0]));
                    }
                }
            }
            if !self.buf.is_empty() {
                return true;
            }
        }
    }
}

impl Iterator for StreamDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.buf.len() && !self.refill() {
            return None;
        }
        let sample = self.buf[self.pos];
        self.pos += 1;
        Some(if crate::AudioSilent::active() { 0.0 } else { sample })
    }
}

impl Source for StreamDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::new(self.channels.max(1)).unwrap_or(NonZero::<u16>::MIN)
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(self.rate).unwrap_or(NonZero::new(44_100).unwrap_or(NonZero::<u32>::MIN))
    }

    fn total_duration(&self) -> Option<Duration> {
        self.length.map(Duration::from_secs_f64)
    }
}

impl Decodable for StreamedMusic {
    type Decoder = StreamDecoder;

    fn decoder(&self) -> StreamDecoder {
        let reader = match open(Arc::clone(&self.bytes), self.ext.as_deref()) {
            Ok(reader) => Some(reader),
            Err(error) => {
                diag::warn!(Audio, "audio: music: stream did not open ({error})");
                None
            }
        };
        StreamDecoder {
            reader,
            buf: Vec::new(),
            pos: 0,
            scratch: None,
            channels: self.channels,
            rate: self.rate,
            length: self.length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 16-bit stereo WAV of `frames` frames at `rate`.
    fn wav(frames: u32, rate: u32) -> Vec<u8> {
        let data = frames * 4;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * 4).to_le_bytes());
        out.extend_from_slice(&4u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data.to_le_bytes());
        for i in 0..frames {
            let v = ((i % 100) as i16 - 50) * 300;
            out.extend_from_slice(&v.to_le_bytes());
            out.extend_from_slice(&(-v).to_le_bytes());
        }
        out
    }

    #[test]
    fn probes_and_streams_a_wav() {
        let bytes: Arc<[u8]> = wav(22_050, 22_050).into();
        let p = probe(Arc::clone(&bytes), Some("wav")).unwrap();
        assert_eq!(p.channels, 2);
        assert_eq!(p.rate, 22_050);
        assert!((p.length.unwrap() - 1.0).abs() < 1e-3, "{p:?}");
        let music = StreamedMusic::new(bytes, Some("wav".into()), &p);
        let samples: Vec<f32> = music.decoder().collect();
        assert_eq!(samples.len(), 22_050 * 2);
    }

    #[test]
    fn garbage_is_not_playable() {
        let bytes: Arc<[u8]> = vec![0u8; 4096].into();
        assert!(probe(bytes, Some("mp3")).is_err());
    }

    /// `IW4L_MUSIC_PROBE=<file>`: probe and fully stream a real music file
    /// (the user's MP3), e.g. `cargo test -p audio probe_real -- --ignored`.
    #[test]
    #[ignore]
    fn probe_real_file() {
        let Some(path) = std::env::var_os("IW4L_MUSIC_PROBE") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
        let bytes: Arc<[u8]> = std::fs::read(&path).unwrap().into();
        let p = probe(Arc::clone(&bytes), ext.as_deref()).unwrap();
        let music = StreamedMusic::new(bytes, ext, &p);
        let n = music.decoder().count();
        let secs = n as f64 / f64::from(p.channels) / f64::from(p.rate);
        eprintln!("{p:?}; streamed {secs:.2} s");
        assert!((secs - p.length.unwrap()).abs() < 0.5);
    }
}
