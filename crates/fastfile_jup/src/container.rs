//! The `.ff` envelope of Modern Warfare III (2023): a 0x120-byte file
//! header, a 0x8000-byte auth region, a 12-byte data header, then the zone
//! image as Oodle blocks of 64 KiB, with a 0x4000-byte master block after
//! every 512 chunks.
//!
//! ```text
//! 0x000 magic "IWffa100"
//! 0x008 u32 0x18                    (same in every retail zone)
//! 0x00c u32 1
//! 0x010 u8[4] flags 01 00 04 xx / 01 00 05 00   (xx varies by zone kind)
//! 0x014 u32 file length - 0x120     (the bytes after this header)
//! 0x038 u64 decompressed zone size
//! 0x040.. u64s not used by the container (block sizes, for the zone layer)
//! 0x0e0 "TAFF" ...
//! 0x120 auth region, 0x8000 bytes:
//!       u8 2, "IWC", "IWffs100", signature bytes, at 0x250 the zone name
//!       (NUL padded), then the first master block
//!       (0x120..0x8000 is otherwise opaque to the container)
//! 0x8120 data header, 12 bytes: u32 ?, u32 decompressed size (low 32
//!        bits of 0x038), u24 block size (0x10000), u8 compressor (6)
//! 0x812c blocks, each:
//!        u32 compressed length, u32 decompressed length, u32 0,
//!        then that many bytes of one Oodle LZ stream,
//!        the next header at the next 4-byte boundary.
//!        Counting the data header as chunk 0, a 0x4000-byte master block
//!        sits before every chunk whose index is a multiple of 512
//!        (i.e. before blocks 511, 1023, 1535, ...).
//! The last block ends exactly at the end of the file.
//! ```
//!
//! Checked on every one of the 16,927 retail `.ff` files of build 1.21
//! (strict walk, no searching, ends at EOF, decompressed sizes sum to
//! 0x038), and fully decoded on 234 of them (6.5 GB of zone, the 15
//! largest included). Every block decodes to 0x10000 bytes except each
//! zone's last.
#![forbid(unsafe_code)]

use crate::oodle::{Decompressor, OodleError, is_oodle_chunk};

pub const MAGIC: &[u8; 8] = b"IWffa100";
pub const AUTH_MAGIC: &[u8; 8] = b"IWffs100";
pub const HEADER_LEN: usize = 0x120;
pub const AUTH_LEN: usize = 0x8000;
pub const DATA_OFFSET: usize = HEADER_LEN + AUTH_LEN;
pub const DATA_HEAD_LEN: usize = 12;
pub const BLOCK_HEAD_LEN: usize = 12;
pub const MASTER_LEN: usize = 0x4000;
/// Chunks (the data header, then the blocks) per master block.
pub const MASTER_RUN: usize = 512;
/// The largest decoded block accepted (an Oodle chunk is at most 256 KiB).
pub const MAX_BLOCK: usize = 0x40000;
/// Offset of the zone name in the auth region, NUL padded.
pub const NAME_OFFSET: usize = 0x250;
pub const NAME_LEN: usize = 0x80;

#[derive(Debug)]
pub enum ContainerError {
    Io(std::io::Error),
    Truncated {
        at: usize,
    },
    BadMagic([u8; 8]),
    /// The data header's size disagrees with the file header's.
    SizeMismatch {
        header: u64,
        data_header: u32,
    },
    BadBlock {
        index: usize,
        offset: usize,
    },
    /// The blocks do not end where the file does.
    Trailing {
        end: usize,
        file_len: usize,
    },
    Oodle {
        index: usize,
        offset: usize,
        error: OodleError,
    },
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "reading zone: {e}"),
            Self::Truncated { at } => write!(f, "zone file ends inside a header at {at:#x}"),
            Self::BadMagic(m) => write!(f, "not a Modern Warfare III zone (magic {m:02x?})"),
            Self::SizeMismatch {
                header,
                data_header,
            } => {
                write!(
                    f,
                    "zone size {header:#x} in the file header, {data_header:#x} in the data header"
                )
            }
            Self::BadBlock { index, offset } => {
                write!(f, "block {index} at {offset:#x} has no valid header")
            }
            Self::Trailing { end, file_len } => {
                write!(f, "blocks end at {end:#x}, file is {file_len:#x} bytes")
            }
            Self::Oodle {
                index,
                offset,
                error,
            } => write!(f, "block {index} at {offset:#x}: {error}"),
        }
    }
}

impl std::error::Error for ContainerError {}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, ContainerError> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
        .ok_or(ContainerError::Truncated { at })
}

fn u64_at(b: &[u8], at: usize) -> Result<u64, ContainerError> {
    b.get(at..at + 8)
        .map(|s| u64::from_le_bytes(s.try_into().unwrap()))
        .ok_or(ContainerError::Truncated { at })
}

/// The parts of the file header and data header the container needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub flags: [u8; 4],
    /// File length minus [`HEADER_LEN`], as stored.
    pub body_len: u32,
    /// Decompressed zone size.
    pub zone_size: u64,
    /// The zone's name from the auth region (empty if absent).
    pub name: String,
    /// The data header's first word (unknown; differs per zone).
    pub data_tag: u32,
    pub block_size: u32,
    pub compressor: u8,
}

pub fn is_jup_zone(file: &[u8]) -> bool {
    file.starts_with(MAGIC)
}

pub fn parse_file_header(file: &[u8]) -> Result<FileHeader, ContainerError> {
    let magic: [u8; 8] = file
        .get(..8)
        .ok_or(ContainerError::Truncated { at: 0 })?
        .try_into()
        .unwrap();
    if &magic != MAGIC {
        return Err(ContainerError::BadMagic(magic));
    }
    let flags: [u8; 4] = file
        .get(0x10..0x14)
        .ok_or(ContainerError::Truncated { at: 0x10 })?
        .try_into()
        .unwrap();
    let body_len = u32_at(file, 0x14)?;
    let zone_size = u64_at(file, 0x38)?;
    let name = file
        .get(NAME_OFFSET..NAME_OFFSET + NAME_LEN)
        .map(|raw| {
            let end = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
            let s = &raw[..end];
            if s.iter().all(|c| c.is_ascii_graphic()) {
                String::from_utf8_lossy(s).into_owned()
            } else {
                String::new()
            }
        })
        .unwrap_or(String::new());
    let data_tag = u32_at(file, DATA_OFFSET)?;
    let data_size = u32_at(file, DATA_OFFSET + 4)?;
    let packed = u32_at(file, DATA_OFFSET + 8)?;
    if data_size as u64 != zone_size & 0xffff_ffff {
        return Err(ContainerError::SizeMismatch {
            header: zone_size,
            data_header: data_size,
        });
    }
    Ok(FileHeader {
        flags,
        body_len,
        zone_size,
        name,
        data_tag,
        block_size: packed & 0x00ff_ffff,
        compressor: (packed >> 24) as u8,
    })
}

/// Where one block's Oodle stream is, and what it decodes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRef {
    /// File offset of the block header.
    pub header: usize,
    /// File offset of the Oodle stream.
    pub data: usize,
    pub compressed: u32,
    pub decompressed: u32,
}

/// Walk the block headers of a whole `.ff` (no decoding). Fails unless the
/// decoded lengths sum to the zone size and the last block ends at EOF.
pub fn walk_blocks(file: &[u8]) -> Result<(FileHeader, Vec<BlockRef>), ContainerError> {
    let header = parse_file_header(file)?;
    let blocks = walk_with(&header, file.len(), |at, buf| {
        let src = file
            .get(at..at + buf.len())
            .ok_or(ContainerError::Truncated { at })?;
        buf.copy_from_slice(src);
        Ok(())
    })?;
    Ok((header, blocks))
}

/// [`walk_blocks`] reading only the headers from a file (13 bytes per
/// block), for checking many zones without loading them.
pub fn walk_blocks_in<R: std::io::Read + std::io::Seek>(
    r: &mut R,
) -> Result<(FileHeader, Vec<BlockRef>), ContainerError> {
    use std::io::SeekFrom;
    let len = r.seek(SeekFrom::End(0))? as usize;
    let mut head = vec![0u8; (DATA_OFFSET + DATA_HEAD_LEN).min(len)];
    r.seek(SeekFrom::Start(0))?;
    r.read_exact(&mut head)?;
    let header = parse_file_header(&head)?;
    let blocks = walk_with(&header, len, |at, buf| {
        if at + buf.len() > len {
            return Err(ContainerError::Truncated { at });
        }
        r.seek(SeekFrom::Start(at as u64))?;
        r.read_exact(buf)?;
        Ok(())
    })?;
    Ok((header, blocks))
}

fn walk_with(
    header: &FileHeader,
    file_len: usize,
    mut read: impl FnMut(usize, &mut [u8]) -> Result<(), ContainerError>,
) -> Result<Vec<BlockRef>, ContainerError> {
    let mut blocks = Vec::new();
    let mut pos = DATA_OFFSET + DATA_HEAD_LEN;
    let mut total: u64 = 0;
    let mut hb = [0u8; BLOCK_HEAD_LEN + 1];
    while total < header.zone_size {
        let index = blocks.len();
        // Chunk index = block index + 1 (the data header is chunk 0).
        if (index + 1) % MASTER_RUN == 0 {
            pos += MASTER_LEN;
        }
        read(pos, &mut hb)?;
        let compressed = u32_at(&hb, 0)?;
        let decompressed = u32_at(&hb, 4)?;
        let zero = u32_at(&hb, 8)?;
        let data = pos + BLOCK_HEAD_LEN;
        if zero != 0
            || compressed == 0
            || compressed as usize > MAX_BLOCK + 2
            || decompressed == 0
            || decompressed as usize > MAX_BLOCK
            || !is_oodle_chunk(hb[BLOCK_HEAD_LEN])
            || data + compressed as usize > file_len
        {
            return Err(ContainerError::BadBlock { index, offset: pos });
        }
        blocks.push(BlockRef {
            header: pos,
            data,
            compressed,
            decompressed,
        });
        total += decompressed as u64;
        pos = (data + compressed as usize + 3) & !3;
    }
    if total != header.zone_size {
        let last = blocks.last().map_or(pos, |b| b.header);
        return Err(ContainerError::BadBlock {
            index: blocks.len().saturating_sub(1),
            offset: last,
        });
    }
    if pos != file_len {
        return Err(ContainerError::Trailing { end: pos, file_len });
    }
    Ok(blocks)
}

/// Decompress a whole `.ff` into its zone image.
pub fn inflate_zone(file: &[u8], oodle: &dyn Decompressor) -> Result<Vec<u8>, ContainerError> {
    let (header, blocks) = walk_blocks(file)?;
    let mut out = vec![0u8; header.zone_size as usize];
    let mut at = 0usize;
    for (index, b) in blocks.iter().enumerate() {
        let src = &file[b.data..b.data + b.compressed as usize];
        let dst = &mut out[at..at + b.decompressed as usize];
        oodle
            .decompress(src, dst)
            .map_err(|error| ContainerError::Oodle {
                index,
                offset: b.header,
                error,
            })?;
        at += b.decompressed as usize;
    }
    Ok(out)
}

/// Read and decompress a `.ff` from disk (opened read-only).
pub fn read_zone(
    path: &std::path::Path,
    oodle: &dyn Decompressor,
) -> Result<(FileHeader, Vec<u8>), ContainerError> {
    let file = std::fs::read(path)?;
    let header = parse_file_header(&file)?;
    Ok((header, inflate_zone(&file, oodle)?))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::oodle::StoredOnly;

    /// A synthetic `.ff` whose blocks are Oodle stored chunks.
    pub(crate) fn synth_ff(zone: &[u8], block: usize) -> Vec<u8> {
        let mut f = vec![0u8; DATA_OFFSET];
        f[..8].copy_from_slice(MAGIC);
        f[8..12].copy_from_slice(&0x18u32.to_le_bytes());
        f[0x10..0x14].copy_from_slice(&[1, 0, 4, 0]);
        f[0x38..0x40].copy_from_slice(&(zone.len() as u64).to_le_bytes());
        f[NAME_OFFSET..NAME_OFFSET + 9].copy_from_slice(b"test_zone");
        f.extend_from_slice(&0x1234_5678u32.to_le_bytes());
        f.extend_from_slice(&(zone.len() as u32).to_le_bytes());
        f.extend_from_slice(&(block as u32 | 6 << 24).to_le_bytes());
        for (i, chunk) in zone.chunks(block).enumerate() {
            if (i + 1) % MASTER_RUN == 0 {
                f.extend(std::iter::repeat_n(0xAAu8, MASTER_LEN));
            }
            f.extend_from_slice(&(chunk.len() as u32 + 2).to_le_bytes());
            f.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            f.extend_from_slice(&0u32.to_le_bytes());
            f.extend_from_slice(&[0xCC, 0x06]);
            f.extend_from_slice(chunk);
            while !f.len().is_multiple_of(4) {
                f.push(0xEE);
            }
        }
        let body = (f.len() - HEADER_LEN) as u32;
        f[0x14..0x18].copy_from_slice(&body.to_le_bytes());
        f
    }

    #[test]
    fn walks_and_inflates_with_padding() {
        let zone: Vec<u8> = (0..1000u32).map(|i| (i * 7 % 251) as u8).collect();
        // 333-byte blocks: every stream ends off a 4-byte boundary.
        let ff = synth_ff(&zone, 333);
        let (h, blocks) = walk_blocks(&ff).unwrap();
        assert_eq!(h.zone_size, 1000);
        assert_eq!(h.name, "test_zone");
        assert_eq!(h.block_size, 333);
        assert_eq!(h.compressor, 6);
        assert_eq!(h.body_len as usize, ff.len() - HEADER_LEN);
        assert_eq!(blocks.len(), 4);
        assert!(blocks.iter().all(|b| b.header % 4 == 0));
        assert_eq!(inflate_zone(&ff, &StoredOnly).unwrap(), zone);
    }

    #[test]
    fn skips_master_blocks() {
        // 1100 blocks: masters before blocks 511 and 1023.
        let zone: Vec<u8> = (0..1100u32 * 8).map(|i| (i ^ (i >> 8)) as u8).collect();
        let ff = synth_ff(&zone, 8);
        let (_, blocks) = walk_blocks(&ff).unwrap();
        assert_eq!(blocks.len(), 1100);
        let (_, seeked) = walk_blocks_in(&mut std::io::Cursor::new(&ff)).unwrap();
        assert_eq!(seeked, blocks);
        assert_eq!(
            blocks[511].header - blocks[510].header,
            8 + 2 + 12 + 2 + MASTER_LEN
        );
        assert_eq!(inflate_zone(&ff, &StoredOnly).unwrap(), zone);
    }

    #[test]
    fn rejects_bad_files() {
        let zone = vec![1u8; 100];
        let mut ff = synth_ff(&zone, 64);
        ff.push(0);
        assert!(matches!(
            walk_blocks(&ff),
            Err(ContainerError::Trailing { .. })
        ));
        let mut ff = synth_ff(&zone, 64);
        ff[0] = b'X';
        assert!(matches!(walk_blocks(&ff), Err(ContainerError::BadMagic(_))));
        let mut ff = synth_ff(&zone, 64);
        ff[DATA_OFFSET + DATA_HEAD_LEN + 8] = 1; // nonzero third header word
        assert!(matches!(
            walk_blocks(&ff),
            Err(ContainerError::BadBlock { index: 0, .. })
        ));
        let mut ff = synth_ff(&zone, 64);
        ff[DATA_OFFSET + 4] ^= 1;
        assert!(matches!(
            walk_blocks(&ff),
            Err(ContainerError::SizeMismatch { .. })
        ));
    }
}
