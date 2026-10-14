//! The `.ff` envelope of Infinite Warfare: `XFileHeader` (with the block
//! sizes and the streamed images' `XStreamFile`s), the signed zones'
//! `IWffs100` auth header and master block, then the zone image as
//! `IWC` LZ4 blocks.
//!
//! ```text
//! 0x00 magic    "IWff0100" (retail, signed) | "IWffu100" (unsigned, tools)
//! 0x08 version  u32 = 0x653 (1619)
//! 0x0c unused u8, hasNoImageFastfile u8, hasNoSharedFastfile u8, unk u8
//! 0x10 fileTimeHigh u32, fileTimeLow u32
//! 0x18 XFileStreamData { u64 size; u64 unk1; u64 unk2; u64 blockSize[10]; u64 unk[8]; }  168 bytes
//! 0xc0 sharedHash u32, sharedCount u32, imageHash u32, imageCount u32
//! 0xd0 (sharedCount + imageCount) x XStreamFile (24 bytes)
//!      fileLen u64, fileLenUnk1 u64, fileLenUnk2 u64
//!      signed: DB_AuthHeader (0x4000, "IWffs100") and DB_MasterBlock (0x4000)
//! data: XFileCompressorHeader { u8 compressor; "IWC" }
//!       XBlockCompressionDataHeader { u64 uncompressedSize; u24 blockSize, u8 type (4 = LZ4) }
//!       blocks { u32 compressed; u64 uncompressed; data } each padded to 4 bytes
//! ```
//!
//! (x64-zt `iw7/zone.cpp` and `iw7/structs.hpp`; read on retail
//! `mp_weapon_world_iw7_m8_tr`: the blocks inflate to exactly the size the
//! data header names, 64 KiB each.) A signed zone's data is checked in
//! 0x4000-byte chunks against its master block; the chunks are plain data,
//! nothing is interleaved below the 8 MiB the master block covers. Past
//! that the data runs on in further master blocks (`dechunk`).

pub const MAGIC_SIGNED: &[u8; 8] = b"IWff0100";
pub const MAGIC_UNSIGNED: &[u8; 8] = b"IWffu100";
pub const MAGIC_AUTH: &[u8; 8] = b"IWffs100";
/// Infinite Warfare PC.
pub const ZONE_VERSION: u32 = 0x653;

pub const STREAM_FILE_LEN: usize = 24;
/// Bytes before the stream table.
pub const HEADER_FIXED_LEN: usize = 0xd0;
/// The auth header and the first master block.
pub const AUTH_LEN: usize = 0x8000;
/// The data a master block covers (512 chunks of 0x4000), and a master
/// block's size.
pub const MASTER_RUN: usize = 512 * 0x4000;
pub const MASTER_LEN: usize = 0x4000;
/// `XFileCompressorHeader` + `XBlockCompressionDataHeader`.
pub const DATA_HEAD_LEN: usize = 16;
/// `XBlockCompressionBlockHeader` (packed).
pub const BLOCK_HEAD_LEN: usize = 12;
pub const BLOCK_COUNT: usize = 10;

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
            Self::BadMagic(m) => write!(f, "not an Infinite Warfare zone (magic {:02x?})", m),
            Self::WrongVersion(v) => write!(f, "zone version {v:#x} is not Infinite Warfare PC ({ZONE_VERSION:#x})"),
            Self::NoAuthHeader => write!(f, "signed zone without an IWffs100 auth header"),
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

/// One streamed image (or shared asset) of a zone (`XStreamFile`): bytes
/// `offset..offset_end` of `imagefile<file_index>.pak`.
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
    pub flags: [u8; 4],
    /// The image's size, and each block's size.
    pub size: u64,
    /// `0x20`: the VIRTUAL bytes the zone's shared streams take ahead of its
    /// own data (close to it: references are offset by about this much).
    pub shared_virtual: u64,
    pub block_sizes: [u64; BLOCK_COUNT],
    pub shared_files: Vec<StreamFile>,
    pub stream_files: Vec<StreamFile>,
    pub file_len: u64,
    /// Where the auth header (signed) or the data (unsigned) begins.
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

/// Is this the start of an Infinite Warfare zone?
pub fn is_iw7_zone(head: &[u8]) -> bool {
    head.len() >= 12 && (&head[..8] == MAGIC_SIGNED || &head[..8] == MAGIC_UNSIGNED) && u32_at(head, 8) == ZONE_VERSION
}

pub fn parse_file_header(file: &[u8]) -> Result<FileHeader, ContainerError> {
    if file.len() < HEADER_FIXED_LEN + 24 {
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
    let mut block_sizes = [0u64; BLOCK_COUNT];
    for (i, b) in block_sizes.iter_mut().enumerate() {
        *b = u64_at(file, 0x30 + 8 * i);
    }
    let shared = u32_at(file, 0xc4) as usize;
    let images = u32_at(file, 0xcc) as usize;
    let table_end = HEADER_FIXED_LEN
        .checked_add((shared + images).checked_mul(STREAM_FILE_LEN).ok_or(ContainerError::Truncated)?)
        .ok_or(ContainerError::Truncated)?;
    if file.len() < table_end + 24 {
        return Err(ContainerError::Truncated);
    }
    let read = |i: usize| {
        let o = HEADER_FIXED_LEN + i * STREAM_FILE_LEN;
        StreamFile {
            word0: u16_at(file, o),
            file_index: u16_at(file, o + 2),
            flags: u32_at(file, o + 4),
            offset: u64_at(file, o + 8),
            offset_end: u64_at(file, o + 16),
        }
    };
    Ok(FileHeader {
        signed,
        version,
        flags: [file[12], file[13], file[14], file[15]],
        size: u64_at(file, 0x18),
        shared_virtual: u64_at(file, 0x20),
        block_sizes,
        shared_files: (0..shared).map(read).collect(),
        stream_files: (shared..shared + images).map(read).collect(),
        file_len: u64_at(file, table_end),
        data_offset: table_end + 24,
    })
}

/// The data of a signed zone with its auth header and master blocks taken
/// out (unsigned: the data as it is).
pub fn dechunk(file: &[u8], header: &FileHeader) -> Result<Vec<u8>, ContainerError> {
    if !header.signed {
        return Ok(file.get(header.data_offset..).ok_or(ContainerError::Truncated)?.to_vec());
    }
    let auth = header.data_offset;
    if file.get(auth..auth + 8) != Some(MAGIC_AUTH.as_slice()) {
        return Err(ContainerError::NoAuthHeader);
    }
    let data = file.get(auth + AUTH_LEN..).unwrap_or(&[]);
    let mut out = Vec::with_capacity(data.len());
    let mut p = 0;
    while p < data.len() {
        let end = (p + MASTER_RUN).min(data.len());
        out.extend_from_slice(&data[p..end]);
        p = end + MASTER_LEN;
    }
    Ok(out)
}

/// The `IWC` LZ4 blocks of the data after `dechunk`, inflated.
pub fn inflate_blocks(data: &[u8]) -> Result<Vec<u8>, ContainerError> {
    if data.len() < DATA_HEAD_LEN || &data[1..4] != b"IWC" {
        return Err(ContainerError::Inflate("no IWC compressor header".into()));
    }
    let total = usize::try_from(u64_at(data, 4)).map_err(|_| ContainerError::Truncated)?;
    if total > 1 << 34 {
        return Err(ContainerError::Inflate(format!("image size {total:#x}")));
    }
    let kind = data[15];
    if kind != 4 && kind != 3 {
        return Err(ContainerError::Inflate(format!("block compression {kind} is not LZ4")));
    }
    let mut out = Vec::with_capacity(total);
    let mut p = DATA_HEAD_LEN;
    while out.len() < total {
        if p + BLOCK_HEAD_LEN > data.len() {
            return Err(ContainerError::Inflate(format!("data ends at block {p:#x} with {:#x} of {total:#x} bytes", out.len())));
        }
        let packed = u32_at(data, p) as usize;
        let size = u64_at(data, p + 4) as usize;
        if size == 0 || size > 1 << 20 {
            return Err(ContainerError::Inflate(format!("block at {p:#x} says {size:#x} bytes")));
        }
        let src = data.get(p + BLOCK_HEAD_LEN..p + BLOCK_HEAD_LEN + packed).ok_or(ContainerError::Truncated)?;
        let at = out.len();
        out.resize(at + size, 0);
        let n = lz4_flex::block::decompress_into(src, &mut out[at..]).map_err(|e| ContainerError::Inflate(format!("block at {p:#x}: {e}")))?;
        if n != size {
            return Err(ContainerError::Inflate(format!("block at {p:#x} inflates to {n:#x}, not {size:#x}")));
        }
        p = (p + BLOCK_HEAD_LEN + packed + 3) & !3;
    }
    out.truncate(total);
    Ok(out)
}

/// Inflates a whole `.ff` file to the zone image.
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
