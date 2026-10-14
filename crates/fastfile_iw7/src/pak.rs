//! The `.pak` files next to the zones: `imagefile<N>.pak` (streamed image
//! levels) and `assetfile<N>.pak` (the zones' shared asset streams).
//!
//! ```text
//! pak header (0x24 bytes)
//!   0x00 magic   "IWffu100" (imagefile, unsigned) | "IWff0100" (assetfile, signed)
//!   0x08 version u32 = 0x653
//!   0x0c ...     (assetfile: u8 1 = signed; imagefile: filetime and 0x10 bytes of hash)
//! then the ranges an `XStreamFile` names (offset..offset_end):
//!   imagefile: an IWC stream: XFileCompressorHeader { u8 2; "IWC" },
//!              XBlockCompressionDataHeader { u64 size; u24 block size; u8 4 = LZ4 },
//!              LZ4 blocks { u32 packed; u64 size; data } padded to 4
//!              (the same envelope as the zone data, `container::inflate_blocks`);
//!              it inflates to that stream level's mip chain, biggest level first.
//!   assetfile: a signed chunk: DB_AuthHeader ("IWffs100", 0x4000), a master
//!              block (0x4000), then the IWC stream (with a further master
//!              block after each 8 MiB, as in a signed zone).
//! ```
//!
//! A zone's shared ranges, inflated and laid end to end in the order of its
//! header's table, are exactly `FileHeader::shared_virtual` bytes: the start
//! of the zone's VIRTUAL block, ahead of the zone's own data.

use crate::container::{self, ContainerError, MAGIC_AUTH, MASTER_LEN, MASTER_RUN, StreamFile};

/// Bytes before the first range of a `.pak`.
pub const PAK_HEADER_LEN: u64 = 0x24;
/// Auth header + first master block ahead of a shared range's data.
pub const SHARED_HEAD_LEN: usize = 0x8000;

/// One `imagefile` range (an IWC stream), inflated.
pub fn inflate_image_range(bytes: &[u8]) -> Result<Vec<u8>, ContainerError> {
    container::inflate_blocks(bytes)
}

/// One `assetfile` range (signed chunk), inflated.
pub fn inflate_shared_range(bytes: &[u8]) -> Result<Vec<u8>, ContainerError> {
    if bytes.get(..8) != Some(MAGIC_AUTH.as_slice()) {
        return Err(ContainerError::NoAuthHeader);
    }
    let data = bytes.get(SHARED_HEAD_LEN..).ok_or(ContainerError::Truncated)?;
    let mut out = Vec::with_capacity(data.len());
    let mut p = 0;
    while p < data.len() {
        let end = (p + MASTER_RUN).min(data.len());
        out.extend_from_slice(&data[p..end]);
        p = end + MASTER_LEN;
    }
    container::inflate_blocks(&out)
}

/// The shared VIRTUAL data of a zone, from its shared ranges' raw bytes laid
/// end to end in table order (e.g. a file of the copied ranges): the inflated
/// data and where each range starts in it.
pub struct SharedVirtual {
    pub data: Vec<u8>,
    /// Per shared range: (VIRTUAL start, inflated length).
    pub ranges: Vec<(u64, u64)>,
}

impl SharedVirtual {
    pub fn from_concatenated(raw: &[u8], files: &[StreamFile]) -> Result<Self, ContainerError> {
        let mut data = Vec::new();
        let mut ranges = Vec::with_capacity(files.len());
        let mut p = 0usize;
        for f in files {
            let n = usize::try_from(f.offset_end - f.offset).map_err(|_| ContainerError::Truncated)?;
            let part = inflate_shared_range(raw.get(p..p + n).ok_or(ContainerError::Truncated)?)?;
            ranges.push((data.len() as u64, part.len() as u64));
            data.extend_from_slice(&part);
            p += n;
        }
        Ok(Self { data, ranges })
    }

    /// Which shared range VIRTUAL offset `v` falls in.
    pub fn range_of(&self, v: u64) -> Option<usize> {
        let i = self.ranges.partition_point(|&(s, _)| s <= v).checked_sub(1)?;
        (v < self.ranges[i].0 + self.ranges[i].1).then_some(i)
    }
}

/// A `.pak` file's range, read from disk.
pub fn read_range(path: &std::path::Path, f: &StreamFile) -> Result<Vec<u8>, ContainerError> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    file.seek(SeekFrom::Start(f.offset))?;
    let mut buf = vec![0u8; usize::try_from(f.offset_end - f.offset).map_err(|_| ContainerError::Truncated)?];
    file.read_exact(&mut buf)?;
    Ok(buf)
}
