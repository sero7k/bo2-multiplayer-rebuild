//! The `.ff` envelope: `XFileHeader`, the image stream table that map zones
//! carry before the data, the `IWffs100` auth header with its hash chunks,
//! and the single zlib stream that inflates to the zone image.
//!
//! ```text
//! 0x00 magic    "IWff0100" (retail, signed) | "IWffu100" (unsigned, tools)
//! 0x08 version  u32 = 0x235
//! 0x0c compress u8 = 1, compressType u8 = 0 (zlib), sizeOfPointer u8 = 8, sizeOfLong u8 = 4
//! 0x10 fileTimeHigh u32, fileTimeLow u32
//! 0x18 imageCount u32
//! 0x1c imageCount x XStreamFile (24 bytes)   -- streamed images in imagefile<N>.pak
//!      baseFileLen u64, totalFileLen u64
//!      signed: "IWffs100" auth header; data = 0x2000-byte chunks from auth+0x4000,
//!              one 0x2000 hash chunk skipped after every 256 data chunks
//!      unsigned: the zlib stream right away
//! ```

use std::io::Read;

pub const MAGIC_SIGNED: &[u8; 8] = b"IWff0100";
pub const MAGIC_UNSIGNED: &[u8; 8] = b"IWffu100";
pub const MAGIC_AUTH: &[u8; 8] = b"IWffs100";
/// Ghosts PC.
pub const ZONE_VERSION: u32 = 0x235;

pub const STREAM_FILE_LEN: usize = 24;
/// Bytes before the stream table.
pub const HEADER_FIXED_LEN: usize = 0x1c;
pub const AUTH_CHUNK_SIZE: usize = 0x2000;
/// Data chunks between two hash chunks.
pub const AUTH_CHUNKS_PER_GROUP: usize = 256;
/// The data starts this far after the auth magic (auth header + first hash chunk).
pub const AUTH_DATA_START: usize = 2 * AUTH_CHUNK_SIZE;

#[derive(Debug)]
pub enum ContainerError {
    Io(std::io::Error),
    Truncated,
    BadMagic([u8; 8]),
    WrongVersion(u32),
    NoAuthHeader,
    Inflate(String),
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "reading zone: {e}"),
            Self::Truncated => write!(f, "zone file is shorter than its header"),
            Self::BadMagic(m) => write!(f, "not an IW6 zone (magic {:02x?})", m),
            Self::WrongVersion(v) => write!(
                f,
                "zone version {v:#x} is not Ghosts PC ({ZONE_VERSION:#x})"
            ),
            Self::NoAuthHeader => write!(f, "signed zone without an IWffs100 auth header"),
            Self::Inflate(e) => write!(f, "zone data does not inflate: {e}"),
        }
    }
}

impl std::error::Error for ContainerError {}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// One streamed image of a map zone (`XStreamFile`): bytes `offset..offset_end`
/// of `imagefile<file_index>.pak`. Only the index and the range are understood;
/// `word0` and `flags` are kept raw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamFile {
    pub word0: u16,
    pub file_index: u16,
    pub flags: u32,
    pub offset: u64,
    pub offset_end: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileHeader {
    pub signed: bool,
    pub version: u32,
    pub compress: u8,
    pub compress_type: u8,
    pub pointer_size: u8,
    pub long_size: u8,
    pub file_time: (u32, u32),
    pub stream_files: Vec<StreamFile>,
    pub base_file_len: u64,
    pub total_file_len: u64,
    /// Where the auth header (signed) or the zlib stream (unsigned) begins.
    pub data_offset: usize,
}

fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}
fn u64_at(b: &[u8], o: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[o..o + 8]);
    u64::from_le_bytes(a)
}

/// Is this the start of a Ghosts zone (either magic, version 0x235)?
pub fn is_iw6_zone(head: &[u8]) -> bool {
    head.len() >= 12
        && (&head[..8] == MAGIC_SIGNED || &head[..8] == MAGIC_UNSIGNED)
        && u32_at(head, 8) == ZONE_VERSION
}

pub fn parse_file_header(file: &[u8]) -> Result<FileHeader, ContainerError> {
    if file.len() < HEADER_FIXED_LEN {
        return Err(ContainerError::Truncated);
    }
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&file[..8]);
    let signed = match &magic {
        m if m == MAGIC_SIGNED => true,
        m if m == MAGIC_UNSIGNED => false,
        _ => return Err(ContainerError::BadMagic(magic)),
    };
    let version = u32_at(file, 8);
    if version != ZONE_VERSION {
        return Err(ContainerError::WrongVersion(version));
    }
    let image_count = u32_at(file, 0x18) as usize;
    let table_end = HEADER_FIXED_LEN
        .checked_add(
            image_count
                .checked_mul(STREAM_FILE_LEN)
                .ok_or(ContainerError::Truncated)?,
        )
        .ok_or(ContainerError::Truncated)?;
    if file.len() < table_end + 16 {
        return Err(ContainerError::Truncated);
    }
    let stream_files = (0..image_count)
        .map(|i| {
            let o = HEADER_FIXED_LEN + i * STREAM_FILE_LEN;
            StreamFile {
                word0: u16_at(file, o),
                file_index: u16_at(file, o + 2),
                flags: u32_at(file, o + 4),
                offset: u64_at(file, o + 8),
                offset_end: u64_at(file, o + 16),
            }
        })
        .collect();
    Ok(FileHeader {
        signed,
        version,
        compress: file[12],
        compress_type: file[13],
        pointer_size: file[14],
        long_size: file[15],
        file_time: (u32_at(file, 0x10), u32_at(file, 0x14)),
        stream_files,
        base_file_len: u64_at(file, table_end),
        total_file_len: u64_at(file, table_end + 8),
        data_offset: table_end + 16,
    })
}

/// The data chunks of a signed zone, hash chunks skipped, as one reader.
struct AuthedChunks<'a> {
    data: &'a [u8],
    pos: usize,
    chunk_left: usize,
    chunks_in_group: usize,
}

impl Read for AuthedChunks<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.pos >= self.data.len() {
                return Ok(0);
            }
            if self.chunk_left == 0 {
                if self.chunks_in_group == AUTH_CHUNKS_PER_GROUP {
                    // A hash chunk.
                    self.pos += AUTH_CHUNK_SIZE;
                    self.chunks_in_group = 0;
                    continue;
                }
                self.chunks_in_group += 1;
                self.chunk_left = AUTH_CHUNK_SIZE;
            }
            let n = buf
                .len()
                .min(self.chunk_left)
                .min(self.data.len() - self.pos);
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            self.chunk_left -= n;
            return Ok(n);
        }
    }
}

/// The joined data chunks of a signed zone (the zlib stream), for tests and tools.
pub fn dechunk(file: &[u8], header: &FileHeader) -> Result<Vec<u8>, ContainerError> {
    let mut out = Vec::new();
    data_reader(file, header)?.read_to_end(&mut out)?;
    Ok(out)
}

fn data_reader<'a>(
    file: &'a [u8],
    header: &FileHeader,
) -> Result<Box<dyn Read + 'a>, ContainerError> {
    if !header.signed {
        return Ok(Box::new(
            file.get(header.data_offset..)
                .ok_or(ContainerError::Truncated)?,
        ));
    }
    let auth = header.data_offset;
    if file.get(auth..auth + 8) != Some(MAGIC_AUTH.as_slice()) {
        return Err(ContainerError::NoAuthHeader);
    }
    let start = auth + AUTH_DATA_START;
    Ok(Box::new(AuthedChunks {
        data: file.get(start..).unwrap_or(&[]),
        pos: 0,
        chunk_left: 0,
        chunks_in_group: 0,
    }))
}

/// Inflates a whole `.ff` file to the zone image (XFile header + blocks' data).
pub fn inflate_zone(file: &[u8]) -> Result<(FileHeader, Vec<u8>), ContainerError> {
    let header = parse_file_header(file)?;
    let reader = data_reader(file, &header)?;
    let mut out = Vec::with_capacity(file.len().saturating_mul(4));
    flate2::read::ZlibDecoder::new(reader)
        .read_to_end(&mut out)
        .map_err(|e| ContainerError::Inflate(e.to_string()))?;
    out.shrink_to_fit();
    Ok((header, out))
}

/// Reads and inflates the zone at `path`.
pub fn open_zone_file(path: &std::path::Path) -> Result<(FileHeader, Vec<u8>), ContainerError> {
    let file = std::fs::read(path)?;
    inflate_zone(&file)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::Write;

    /// A signed zone around `image` (zlib, chunked with hash chunks).
    pub(crate) fn signed_zone(image: &[u8], images: &[StreamFile]) -> Vec<u8> {
        let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        z.write_all(image).unwrap();
        let stream = z.finish().unwrap();
        let mut f = Vec::new();
        f.extend_from_slice(MAGIC_SIGNED);
        f.extend_from_slice(&ZONE_VERSION.to_le_bytes());
        f.extend_from_slice(&[1, 0, 8, 4]);
        f.extend_from_slice(&[0; 8]);
        f.extend_from_slice(&(images.len() as u32).to_le_bytes());
        for s in images {
            f.extend_from_slice(&s.word0.to_le_bytes());
            f.extend_from_slice(&s.file_index.to_le_bytes());
            f.extend_from_slice(&s.flags.to_le_bytes());
            f.extend_from_slice(&s.offset.to_le_bytes());
            f.extend_from_slice(&s.offset_end.to_le_bytes());
        }
        f.extend_from_slice(&0u64.to_le_bytes());
        f.extend_from_slice(&0u64.to_le_bytes());
        let auth = f.len();
        f.extend_from_slice(MAGIC_AUTH);
        f.resize(auth + AUTH_DATA_START, 0xAA);
        for (i, chunk) in stream.chunks(AUTH_CHUNK_SIZE).enumerate() {
            if i > 0 && i % AUTH_CHUNKS_PER_GROUP == 0 {
                f.extend(std::iter::repeat_n(0x55u8, AUTH_CHUNK_SIZE));
            }
            f.extend_from_slice(chunk);
        }
        f
    }

    fn noise(n: usize) -> Vec<u8> {
        let mut x = 0x1234_5678u32;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x as u8
            })
            .collect()
    }

    #[test]
    fn header_and_stream_table() {
        let images = [StreamFile {
            word0: 0,
            file_index: 59,
            flags: 1,
            offset: 10,
            offset_end: 30,
        }];
        let f = signed_zone(b"hello", &images);
        let h = parse_file_header(&f).unwrap();
        assert!(h.signed);
        assert_eq!(
            (h.compress, h.compress_type, h.pointer_size, h.long_size),
            (1, 0, 8, 4)
        );
        assert_eq!(h.stream_files, images);
        assert_eq!(h.data_offset, HEADER_FIXED_LEN + STREAM_FILE_LEN + 16);
        assert!(is_iw6_zone(&f));
    }

    #[test]
    fn inflates_across_hash_chunks() {
        // Incompressible, so the stream spans more than 256 chunks.
        let image = noise(AUTH_CHUNK_SIZE * 300);
        let f = signed_zone(&image, &[]);
        let (_, out) = inflate_zone(&f).unwrap();
        assert_eq!(out, image);
    }

    #[test]
    fn rejects_other_games() {
        let mut f = signed_zone(b"x", &[]);
        f[8] = 0x14;
        f[9] = 0x01;
        assert!(matches!(
            parse_file_header(&f),
            Err(ContainerError::WrongVersion(0x114))
        ));
        assert!(matches!(
            parse_file_header(b"TAff0100xxxxxxxxxxxxxxxxxxxxxxxx"),
            Err(ContainerError::BadMagic(_))
        ));
    }
}
