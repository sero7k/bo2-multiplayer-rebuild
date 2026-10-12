//! World at War `.ff` fastfiles: the same container as CoD4's (`IWffu100`, a
//! version number and one zlib stream), version 387 (0x183).

use anyhow::{Context, Result, bail};
use flate2::read::ZlibDecoder;
use std::io::Read;
use std::path::Path;

pub const VERSION: u32 = 0x183;

/// Read a fastfile from disk and return its decompressed zone data.
pub fn load(path: &Path) -> Result<Vec<u8>> {
    let raw = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    decompress(&raw).with_context(|| format!("decompressing {}", path.display()))
}

pub fn decompress(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() < 12 {
        bail!("file too small to be a fastfile");
    }
    if &raw[..8] != iw3::fastfile::MAGIC_UNSIGNED {
        bail!("bad magic {:?} (signed/console fastfiles are not supported)", &raw[..8]);
    }
    let version = u32::from_le_bytes(raw[8..12].try_into().unwrap());
    if version != VERSION {
        bail!("unsupported fastfile version {version:#x} (expected {VERSION:#x}, PC World at War)");
    }
    let mut out = Vec::with_capacity(raw.len().saturating_mul(2));
    ZlibDecoder::new(&raw[12..]).read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    pub(crate) fn wrap(zone: &[u8]) -> Vec<u8> {
        let mut out = iw3::fastfile::MAGIC_UNSIGNED.to_vec();
        out.extend_from_slice(&VERSION.to_le_bytes());
        let mut z = ZlibEncoder::new(Vec::new(), Compression::fast());
        z.write_all(zone).unwrap();
        out.extend(z.finish().unwrap());
        out
    }

    #[test]
    fn round_trip() {
        let zone = b"hello world at war".repeat(10);
        assert_eq!(decompress(&wrap(&zone)).unwrap(), zone);
    }

    #[test]
    fn rejects_cod4_and_garbage() {
        let mut cod4 = wrap(b"x");
        cod4[8..12].copy_from_slice(&5u32.to_le_bytes());
        assert!(decompress(&cod4).unwrap_err().to_string().contains("version"));
        assert!(decompress(b"IWffu100").is_err());
        assert!(decompress(b"IWff0100\x83\x01\x00\x00zz").is_err());
        let mut truncated = wrap(&[7u8; 4096]);
        truncated.truncate(20);
        // A cut stream gives what it holds (or an error); the zone parser
        // then stops where the data ends.
        let _ = decompress(&truncated);
    }
}
