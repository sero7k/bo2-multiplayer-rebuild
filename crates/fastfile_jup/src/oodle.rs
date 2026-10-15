//! Oodle backends for the `.ff` blocks and the `.xsub` entries.
//!
//! Every compressed run in Modern Warfare III is an Oodle LZ stream. The
//! zones' chunks say compressor 6 (Kraken in the open decoders' numbering);
//! all the Oodle codecs share the chunk framing. The repository carries no Oodle decoder: no pure-Rust one with
//! a licence that fits this Apache-2.0 tree exists on crates.io (`oozle` is
//! GPL-3.0; `oozextract` says MIT but is a structural port of GPL-3.0
//! `ooz`). So decoding goes through [`Decompressor`], and the backends are:
//!
//! - [`StoredOnly`], pure Rust: the Oodle "stored" chunks (first byte
//!   `0xCC`/`0x4C`, the raw bytes after a two-byte header) and nothing else.
//!   Enough for synthetic test data and the incompressible blocks.
//! - `OodleDll` (Windows only, [`crate::oodle_dll`]): the game's own
//!   `oo2core_8_win64.dll`, loaded at runtime from the owner's install. The
//!   DLL is never shipped or committed.
//!
//! A pure decoder can replace the DLL later by implementing the trait.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// The DLL name next to `cod23-cod.exe` in the install root.
pub const OODLE_DLL_NAME: &str = "oo2core_8_win64.dll";
/// Overrides where the DLL is looked for (a full path to the DLL).
pub const OODLE_DLL_ENV: &str = "JUP_OODLE_DLL";

#[derive(Debug)]
pub enum OodleError {
    /// The backend cannot decode this stream (e.g. [`StoredOnly`] given a
    /// Leviathan stream).
    Unsupported(&'static str),
    /// The decoder ran and produced a different length (or failed: 0).
    WrongLength { got: i64, want: usize },
    /// The first byte is not an Oodle chunk header (`low nibble 0xC`).
    NotOodle(u8),
    /// No backend could be set up (the DLL is missing, or not on Windows).
    NoBackend(String),
}

impl std::fmt::Display for OodleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(why) => {
                write!(f, "Oodle stream not supported by this backend: {why}")
            }
            Self::WrongLength { got, want } => {
                write!(f, "Oodle decoded {got} bytes, expected {want}")
            }
            Self::NotOodle(b) => write!(f, "not an Oodle stream (first byte {b:#04x})"),
            Self::NoBackend(why) => write!(f, "no Oodle backend: {why}"),
        }
    }
}

impl std::error::Error for OodleError {}

/// Decodes one Oodle LZ stream whose decoded length is known.
pub trait Decompressor: Send + Sync {
    fn name(&self) -> &str;
    /// Fill all of `dst` from `src`; anything else is an error.
    fn decompress(&self, src: &[u8], dst: &mut [u8]) -> Result<(), OodleError>;
}

/// The two header bytes of an Oodle LZ chunk: `0x0C` in the low nibble,
/// bit 6 = stored (uncompressed), bit 7 = decoder reset; the second byte is
/// the compressor (`6` Kraken, `12` Leviathan, ...) with bit 7 = per-chunk CRC present.
pub fn is_oodle_chunk(first: u8) -> bool {
    first & 0x0f == 0x0c && (first >> 4) & 3 == 0
}

/// Oodle's compressor id from the second header byte.
pub fn compressor_name(id: u8) -> &'static str {
    match id & 0x7f {
        5 => "lzna",
        6 => "kraken",
        10 => "mermaid/selkie",
        11 => "bitknit",
        12 => "leviathan",
        _ => "other",
    }
}

/// Pure-Rust backend for stored chunks only.
#[derive(Debug, Default, Clone, Copy)]
pub struct StoredOnly;

impl Decompressor for StoredOnly {
    fn name(&self) -> &str {
        "stored-only"
    }

    fn decompress(&self, src: &[u8], dst: &mut [u8]) -> Result<(), OodleError> {
        // One stored chunk covers up to 256 KiB of output; the zone blocks
        // and the xsub entries checked are at most that.
        let first = *src.first().ok_or(OodleError::Unsupported("empty stream"))?;
        if !is_oodle_chunk(first) {
            return Err(OodleError::NotOodle(first));
        }
        if first & 0x40 == 0 {
            return Err(OodleError::Unsupported(
                "compressed chunk (needs the Oodle DLL)",
            ));
        }
        if src.len() < 2 || src[1] & 0x80 != 0 {
            return Err(OodleError::Unsupported("stored chunk with a CRC"));
        }
        if dst.len() > 0x40000 {
            return Err(OodleError::Unsupported(
                "stored stream of more than one chunk",
            ));
        }
        let body = &src[2..];
        if body.len() != dst.len() {
            return Err(OodleError::WrongLength {
                got: body.len() as i64,
                want: dst.len(),
            });
        }
        dst.copy_from_slice(body);
        Ok(())
    }
}

/// Where the game's DLL would be: `$JUP_OODLE_DLL`, else `<install>/oo2core_8_win64.dll`.
pub fn dll_path(install: &Path) -> PathBuf {
    match std::env::var_os(OODLE_DLL_ENV) {
        Some(p) => PathBuf::from(p),
        None => install.join(OODLE_DLL_NAME),
    }
}

/// The best backend available: the game's DLL on Windows, else an error
/// (callers may fall back to [`StoredOnly`] to at least walk the layout).
pub fn load(install: &Path) -> Result<Box<dyn Decompressor>, OodleError> {
    #[cfg(windows)]
    {
        let dll = crate::oodle_dll::OodleDll::load(&dll_path(install))?;
        Ok(Box::new(dll))
    }
    #[cfg(not(windows))]
    {
        let _ = install;
        Err(OodleError::NoBackend(
            "the game's Oodle DLL loads only on Windows and no pure-Rust decoder is wired in"
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_chunk_round_trips() {
        let mut src = vec![0xCC, 0x06];
        src.extend_from_slice(b"hello zone");
        let mut dst = [0u8; 10];
        StoredOnly.decompress(&src, &mut dst).unwrap();
        assert_eq!(&dst, b"hello zone");
    }

    #[test]
    fn stored_only_refuses_compressed_and_junk() {
        let mut dst = [0u8; 4];
        assert!(matches!(
            StoredOnly.decompress(&[0x8C, 0x06, 1, 2, 3], &mut dst),
            Err(OodleError::Unsupported(_))
        ));
        assert!(matches!(
            StoredOnly.decompress(&[0x12, 0, 0, 0, 0, 0], &mut dst),
            Err(OodleError::NotOodle(0x12))
        ));
        assert!(matches!(
            StoredOnly.decompress(&[0xCC, 0x06, 1, 2, 3], &mut dst),
            Err(OodleError::WrongLength { got: 3, want: 4 })
        ));
    }
}
