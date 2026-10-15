//! The decompressed zone as the map reader sees it: the asset list, the
//! name hash, little-endian reads, and back-pointers to other assets.
//!
//! (A minimal private reading of the zone header; the zone layer proper
//! replaces it once it lands.)

/// "Data follows" pointer (`-2`).
pub(crate) const FOLLOW: u64 = 0xFFFF_FFFF_FFFF_FFFE;

/// Modern Warfare III's asset name hash: FNV-1a 64 from its own offset
/// basis over the lower-cased name (`\` as `/`), the top bit cleared.
pub fn name_hash(name: &str) -> u64 {
    let mut h: u64 = 0x47F5_817A_5EF9_61BA;
    for b in name.bytes() {
        let b = if b == b'\\' {
            b'/'
        } else {
            b.to_ascii_lowercase()
        };
        h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h & 0x7FFF_FFFF_FFFF_FFFF
}

pub(crate) fn u16_at(b: &[u8], o: usize) -> u16 {
    b.get(o..o + 2)
        .map_or(0, |s| u16::from_le_bytes([s[0], s[1]]))
}
pub(crate) fn u32_at(b: &[u8], o: usize) -> u32 {
    b.get(o..o + 4)
        .map_or(0, |s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}
pub(crate) fn u64_at(b: &[u8], o: usize) -> u64 {
    b.get(o..o + 8)
        .map_or(0, |s| u64::from_le_bytes(s.try_into().unwrap()))
}
pub(crate) fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_bits(u32_at(b, o))
}
pub(crate) fn v3_at(b: &[u8], o: usize) -> [f32; 3] {
    [f32_at(b, o), f32_at(b, o + 4), f32_at(b, o + 8)]
}

/// Every place `needle` occurs.
pub(crate) fn find_all(hay: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    if needle.is_empty() || hay.len() < needle.len() {
        return out;
    }
    let last = hay.len() - needle.len();
    let mut i = 0;
    while i <= last {
        match hay[i..=last].iter().position(|&b| b == needle[0]) {
            Some(p) => {
                let at = i + p;
                if &hay[at..at + needle.len()] == needle {
                    out.push(at);
                }
                i = at + 1;
            }
            None => break,
        }
    }
    out
}

/// A decompressed zone: its bytes and asset list.
pub struct ZoneImage {
    pub bytes: Vec<u8>,
    /// Each asset's type.
    pub types: Vec<u32>,
    /// Where the asset list starts in the image.
    pub list_at: usize,
    /// Where the first asset's data starts.
    pub data_at: usize,
}

impl ZoneImage {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, String> {
        let strings = u64_at(&bytes, 0) as usize;
        let strings_ptr = u64_at(&bytes, 8);
        let count = u64_at(&bytes, 0x10) as usize;
        let list_ptr = u64_at(&bytes, 0x18);
        if count > 4_000_000 || strings > 4_000_000 {
            return Err(format!(
                "zone header counts out of range ({strings} strings, {count} assets)"
            ));
        }
        let mut o = 0x20usize;
        if strings_ptr == FOLLOW {
            let ptrs: Vec<u64> = (0..strings).map(|k| u64_at(&bytes, o + 8 * k)).collect();
            o += 8 * strings;
            for p in ptrs {
                if p == FOLLOW {
                    let end = bytes
                        .get(o..)
                        .and_then(|r| r.iter().position(|&b| b == 0))
                        .ok_or("script string runs off the zone")?;
                    o += end + 1;
                }
            }
        }
        if list_ptr != FOLLOW {
            return Err(format!("asset list pointer {list_ptr:#x}"));
        }
        let list_at = o;
        let data_at = o + 16 * count;
        if data_at > bytes.len() {
            return Err("asset list runs off the zone".into());
        }
        let types = (0..count)
            .map(|k| u64_at(&bytes, o + 16 * k) as u32)
            .collect();
        Ok(Self {
            bytes,
            types,
            list_at,
            data_at,
        })
    }

    /// The asset a back-pointer names. Block 15 starts with the asset
    /// list, so a pointer to another asset of the zone is a pointer to its
    /// entry there (0x14 bytes ahead of where the image has the list).
    pub fn asset_of(&self, ptr: u64) -> Option<usize> {
        if ptr >> 32 != 0x0F || ptr & 1 == 0 {
            return None;
        }
        let off = (ptr & 0xFFFF_FFFF) as usize - 1;
        let base = self.list_at.checked_sub(0x14)?;
        let rel = off.checked_sub(base)?;
        (rel % 16 == 0 && rel / 16 < self.types.len()).then_some(rel / 16)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash_matches_the_zones() {
        assert_eq!(super::name_hash("mp_jup_rust"), 0x45ac_400e_a6d2_ee52);
        assert_eq!(
            super::name_hash("JUP_WEAPON/JUP_JP01_AR_GOLF36"),
            0x0d38_211c_17a1_4b64
        );
    }
}
