//! Streamed sounds: `zone/soundfile<N>.pak` (and, for localized sounds,
//! `zone/<language>/<lang>_soundfile<N>.pak`). The paks have no table of
//! their own: they are FLAC streams (`fLaC` + STREAMINFO ...) laid end to
//! end on 0x800 boundaries, and an alias's `StreamFileName` gives the pak
//! (`fileIndex`), the byte offset and the length.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::sound::StreamedSound;

pub const FLAC_MAGIC: &[u8; 4] = b"fLaC";
/// The biggest stream read into memory (the longest retail track is ~45 MB).
pub const MAX_STREAM_BYTES: u64 = 256 * 1024 * 1024;

/// The three-letter prefix of a language folder's files (`english` -> `eng`).
pub fn language_prefix(language: &str) -> String {
    match language.to_ascii_lowercase().as_str() {
        "english" => "eng".into(),
        "french" => "fre".into(),
        "german" => "ger".into(),
        "italian" => "ita".into(),
        "spanish" => "spa".into(),
        "russian" => "rus".into(),
        "polish" => "pol".into(),
        "portuguese" => "por".into(),
        "japanese" => "jap".into(),
        "korean" => "kor".into(),
        "traditional_chinese" | "chinese" => "chi".into(),
        other => other.chars().take(3).collect(),
    }
}

/// The pak holding `stream` under `zone_dir` (`<game>/zone`); `None` for a
/// loose file (`file_index` 0). Localized streams are looked up in the
/// `language` folder (unverified on retail data: no localized stream has
/// been seen in the MP zones read so far).
pub fn sound_pak_path(zone_dir: &Path, stream: &StreamedSound, language: &str) -> Option<PathBuf> {
    if stream.file_index == 0 {
        return None;
    }
    let n = stream.file_index;
    Some(if stream.localized {
        zone_dir
            .join(language)
            .join(format!("{}_soundfile{n}.pak", language_prefix(language)))
    } else {
        zone_dir.join(format!("soundfile{n}.pak"))
    })
}

/// Reads a stream's bytes from its pak.
pub fn read_stream(
    zone_dir: &Path,
    stream: &StreamedSound,
    language: &str,
) -> std::io::Result<Vec<u8>> {
    use std::io::{Error, ErrorKind};
    let path = sound_pak_path(zone_dir, stream, language).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "loose (file index 0) streams are not in a pak",
        )
    })?;
    if stream.length == 0 || stream.length > MAX_STREAM_BYTES {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("stream length {} out of range", stream.length),
        ));
    }
    let mut file = std::fs::File::open(&path)
        .map_err(|e| Error::new(e.kind(), format!("{}: {e}", path.display())))?;
    let size = file.metadata()?.len();
    if stream.offset.saturating_add(stream.length) > size {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "{}: {:#x}+{:#x} is past its end ({size:#x})",
                path.display(),
                stream.offset,
                stream.length
            ),
        ));
    }
    file.seek(SeekFrom::Start(stream.offset))?;
    let mut out = vec![0u8; stream.length as usize];
    file.read_exact(&mut out)?;
    Ok(out)
}

/// The first bytes of a stream (to check its magic without reading it all).
pub fn peek_stream(
    zone_dir: &Path,
    stream: &StreamedSound,
    language: &str,
    n: usize,
) -> std::io::Result<Vec<u8>> {
    let path = sound_pak_path(zone_dir, stream, language)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "loose stream"))?;
    let mut file = std::fs::File::open(&path)?;
    file.seek(SeekFrom::Start(stream.offset))?;
    let mut out = vec![0u8; n.min(stream.length as usize)];
    file.read_exact(&mut out)?;
    Ok(out)
}

/// A FLAC stream's STREAMINFO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlacInfo {
    pub sample_rate: u32,
    pub channels: u8,
    pub bits_per_sample: u8,
    pub total_samples: u64,
}

impl FlacInfo {
    pub fn seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.total_samples as f64 / f64::from(self.sample_rate)
        }
    }
}

/// STREAMINFO of a FLAC stream (the first metadata block must be it).
pub fn flac_info(bytes: &[u8]) -> Option<FlacInfo> {
    if bytes.get(..4)? != FLAC_MAGIC || bytes.get(4)? & 0x7F != 0 {
        return None;
    }
    let si = bytes.get(8..8 + 34)?;
    let mut v = [0u8; 8];
    v.copy_from_slice(&si[10..18]);
    let v = u64::from_be_bytes(v);
    Some(FlacInfo {
        sample_rate: (v >> 44) as u32,
        channels: (((v >> 41) & 7) + 1) as u8,
        bits_per_sample: (((v >> 36) & 31) + 1) as u8,
        total_samples: v & ((1 << 36) - 1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The STREAMINFO of `music_mainmenu_mp` (soundfile1.pak @ 0x0bbad000).
    const MAINMENU_HEAD: [u8; 42] = [
        0x66, 0x4c, 0x61, 0x43, 0x00, 0x00, 0x00, 0x22, 0x04, 0x00, 0x04, 0x00, 0x00, 0x01, 0xf2,
        0x00, 0x0a, 0x0a, 0x0a, 0xc4, 0x42, 0xf0, 0x00, 0x58, 0x37, 0x4e, 0xae, 0x79, 0x2a, 0x39,
        0x33, 0xef, 0x8e, 0x40, 0x46, 0x98, 0x2f, 0xdd, 0xb3, 0x96, 0x66, 0xe1,
    ];

    #[test]
    fn reads_streaminfo() {
        let info = flac_info(&MAINMENU_HEAD).unwrap();
        assert_eq!(
            (info.sample_rate, info.channels, info.bits_per_sample),
            (44_100, 2, 16)
        );
        assert_eq!(info.total_samples, 5_781_326);
        assert!((info.seconds() - 131.0958).abs() < 1e-3);
        assert!(flac_info(b"RIFF....").is_none());
    }

    #[test]
    fn pak_paths() {
        let s = StreamedSound {
            localized: false,
            file_index: 1,
            offset: 0,
            length: 1,
            total_msec: 0,
            raw_dir: None,
            raw_name: None,
        };
        let zone = Path::new("/g/ghosts/zone");
        assert_eq!(
            sound_pak_path(zone, &s, "english").unwrap(),
            zone.join("soundfile1.pak")
        );
        let loc = StreamedSound {
            localized: true,
            file_index: 6,
            ..s.clone()
        };
        assert_eq!(
            sound_pak_path(zone, &loc, "english").unwrap(),
            zone.join("english/eng_soundfile6.pak")
        );
        let loose = StreamedSound { file_index: 0, ..s };
        assert!(sound_pak_path(zone, &loose, "english").is_none());
    }

    #[test]
    fn reads_a_range() {
        let dir = std::env::temp_dir().join(format!("iw6pak{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut pak = vec![0u8; 0x800];
        pak.extend_from_slice(&MAINMENU_HEAD);
        std::fs::write(dir.join("soundfile3.pak"), &pak).unwrap();
        let s = StreamedSound {
            localized: false,
            file_index: 3,
            offset: 0x800,
            length: 42,
            total_msec: 131_095,
            raw_dir: None,
            raw_name: None,
        };
        let bytes = read_stream(&dir, &s, "english").unwrap();
        assert_eq!(&bytes[..4], FLAC_MAGIC);
        assert_eq!(peek_stream(&dir, &s, "english", 4).unwrap(), FLAC_MAGIC);
        let past = StreamedSound { offset: 0x900, ..s };
        assert!(read_stream(&dir, &past, "english").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
