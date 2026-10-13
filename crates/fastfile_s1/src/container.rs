//! The `.ff` envelope of Advanced Warfare: `XFileHeader`, the image stream
//! table, the `S1ffs100` auth header, then the zone image as LZ4 blocks.
//!
//! ```text
//! 0x00 magic    "S1ff0100" (retail, signed) | "S1ffu100" (unsigned, tools)
//! 0x08 version  u32 = 0x72E
//! 0x0c compress u8 = 1, compressType u8 = 4 (LZ4), sizeOfPointer u8 = 8, sizeOfLong u8 = 4
//! 0x10 fileTimeHigh u32, fileTimeLow u32
//! 0x18 imageCount u32
//! 0x1c imageCount x XStreamFile (24 bytes)
//!      baseFileLen u64, totalFileLen u64
//!      signed: "S1ffs100" auth header; the data from auth+0x8000 in
//!              0x800000-byte runs, each followed by a 0x4000-byte hash block
//! data: u64 image size, u32 (flags); then blocks
//!       { u32 compressed; u32 inflated; LZ4 block; } each padded to 4 bytes
//! ```
//!
//! (Read on retail `common_mp`, `mp_vm_*_tr`: the blocks inflate to exactly
//! the image size the data starts with; blocks inflate to 64 KiB.)

pub const MAGIC_SIGNED: &[u8; 8] = b"S1ff0100";
pub const MAGIC_UNSIGNED: &[u8; 8] = b"S1ffu100";
pub const MAGIC_AUTH: &[u8; 8] = b"S1ffs100";
/// Advanced Warfare PC.
pub const ZONE_VERSION: u32 = 0x72E;

pub const STREAM_FILE_LEN: usize = 24;
/// Bytes before the stream table.
pub const HEADER_FIXED_LEN: usize = 0x1c;
/// The data starts this far after the auth magic.
pub const AUTH_DATA_START: usize = 0x8000;
/// Data bytes between two hash blocks, and a hash block's size.
pub const AUTH_RUN: usize = 0x80_0000;
pub const AUTH_HASH_LEN: usize = 0x4000;
/// Bytes before the first LZ4 block (image size u64, flags u32).
pub const DATA_HEAD_LEN: usize = 12;

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
            Self::BadMagic(m) => write!(f, "not an AW zone (magic {:02x?})", m),
            Self::WrongVersion(v) => write!(
                f,
                "zone version {v:#x} is not Advanced Warfare PC ({ZONE_VERSION:#x})"
            ),
            Self::NoAuthHeader => write!(f, "signed zone without an S1ffs100 auth header"),
            Self::Inflate(e) => write!(f, "zone data does not decompress: {e}"),
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

/// Is this the start of an AW zone (either magic, version 0x72E)?
pub fn is_s1_zone(head: &[u8]) -> bool {
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

/// The data of a signed zone with its hash blocks taken out (unsigned: the
/// data as it is).
pub fn dechunk(file: &[u8], header: &FileHeader) -> Result<Vec<u8>, ContainerError> {
    if !header.signed {
        return Ok(file
            .get(header.data_offset..)
            .ok_or(ContainerError::Truncated)?
            .to_vec());
    }
    let auth = header.data_offset;
    if file.get(auth..auth + 8) != Some(MAGIC_AUTH.as_slice()) {
        return Err(ContainerError::NoAuthHeader);
    }
    let data = file.get(auth + AUTH_DATA_START..).unwrap_or(&[]);
    let mut out = Vec::with_capacity(data.len());
    let mut p = 0;
    while p < data.len() {
        let end = (p + AUTH_RUN).min(data.len());
        out.extend_from_slice(&data[p..end]);
        p = end + AUTH_HASH_LEN;
    }
    Ok(out)
}

/// The LZ4 blocks of the data after `dechunk`, inflated.
pub fn inflate_blocks(data: &[u8]) -> Result<Vec<u8>, ContainerError> {
    if data.len() < DATA_HEAD_LEN {
        return Err(ContainerError::Truncated);
    }
    let total = usize::try_from(u64_at(data, 0)).map_err(|_| ContainerError::Truncated)?;
    if total > 1 << 34 {
        return Err(ContainerError::Inflate(format!("image size {total:#x}")));
    }
    let mut out = Vec::with_capacity(total);
    let mut p = DATA_HEAD_LEN;
    while out.len() < total {
        if p + 8 > data.len() {
            return Err(ContainerError::Inflate(format!(
                "data ends at block {p:#x} with {:#x} of {total:#x} bytes",
                out.len()
            )));
        }
        let packed = u32_at(data, p) as usize;
        let size = u32_at(data, p + 4) as usize;
        let src = data
            .get(p + 8..p + 8 + packed)
            .ok_or(ContainerError::Truncated)?;
        let at = out.len();
        out.resize(at + size, 0);
        let n = lz4_flex::block::decompress_into(src, &mut out[at..])
            .map_err(|e| ContainerError::Inflate(format!("block at {p:#x}: {e}")))?;
        if n != size {
            return Err(ContainerError::Inflate(format!(
                "block at {p:#x} inflates to {n:#x}, not {size:#x}"
            )));
        }
        p = (p + 8 + packed + 3) & !3;
    }
    out.truncate(total);
    Ok(out)
}

/// Inflates a whole `.ff` file to the zone image (XFile header + blocks' data).
pub fn inflate_zone(file: &[u8]) -> Result<(FileHeader, Vec<u8>), ContainerError> {
    let header = parse_file_header(file)?;
    let data = dechunk(file, &header)?;
    let out = inflate_blocks(&data)?;
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

    /// A signed zone around `image`: LZ4 blocks of 64 KiB, hash blocks
    /// after every run.
    pub(crate) fn signed_zone(image: &[u8], images: &[StreamFile]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(image.len() as u64).to_le_bytes());
        data.extend_from_slice(&[0, 0, 1, 4]);
        for chunk in image.chunks(0x10000) {
            let packed = lz4_flex::block::compress(chunk);
            data.extend_from_slice(&(packed.len() as u32).to_le_bytes());
            data.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            data.extend_from_slice(&packed);
            while data.len() % 4 != 0 {
                data.push(0);
            }
        }
        let mut f = Vec::new();
        f.extend_from_slice(MAGIC_SIGNED);
        f.extend_from_slice(&ZONE_VERSION.to_le_bytes());
        f.extend_from_slice(&[1, 4, 8, 4]);
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
        for (i, run) in data.chunks(AUTH_RUN).enumerate() {
            if i > 0 {
                f.extend(std::iter::repeat_n(0x55u8, AUTH_HASH_LEN));
            }
            f.extend_from_slice(run);
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
            (1, 4, 8, 4)
        );
        assert_eq!(h.stream_files, images);
        assert_eq!(h.data_offset, HEADER_FIXED_LEN + STREAM_FILE_LEN + 16);
        assert!(is_s1_zone(&f));
        assert_eq!(inflate_zone(&f).unwrap().1, b"hello");
    }

    #[test]
    fn inflates_across_hash_blocks() {
        // Incompressible, so the data spans more than one run.
        let image = noise(AUTH_RUN + 0x30000);
        let f = signed_zone(&image, &[]);
        let (_, out) = inflate_zone(&f).unwrap();
        assert_eq!(out, image);
    }

    #[test]
    fn rejects_other_games() {
        let mut f = signed_zone(b"x", &[]);
        f[8] = 0x35;
        f[9] = 0x02;
        assert!(matches!(
            parse_file_header(&f),
            Err(ContainerError::WrongVersion(0x235))
        ));
        assert!(matches!(
            parse_file_header(b"IWff0100xxxxxxxxxxxxxxxxxxxxxxxx"),
            Err(ContainerError::BadMagic(_))
        ));
    }
}
