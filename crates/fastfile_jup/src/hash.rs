//! Modern Warfare III's asset name hash: 64-bit FNV-1a with its own offset
//! basis, over the name lower-cased with `\` as `/`, top bit cleared.
//!
//! Zones store most asset names only as this hash. A stored hash with its
//! top bit set is a reference to an asset defined in another zone (see
//! [`crate::zone`]). Checked against names found in the zones (weapons,
//! models, localize keys, zone names).
#![forbid(unsafe_code)]

/// The offset basis (FNV's own is 0xcbf29ce484222325).
pub const BASIS: u64 = 0x47f5_817a_5ef9_61ba;
/// FNV's 64-bit prime.
pub const PRIME: u64 = 0x0000_0100_0000_01b3;
/// What a name hash keeps (the top bit marks a reference).
pub const MASK: u64 = 0x7fff_ffff_ffff_ffff;
/// The reference bit of a stored hash.
pub const REF_BIT: u64 = 1 << 63;

/// The hash of an asset name (`jup_jp01_ar_golf36_mp`).
#[must_use]
pub fn name(s: &str) -> u64 {
    let mut h = BASIS;
    for b in s.bytes() {
        let b = match b {
            b'\\' => b'/',
            b => b.to_ascii_lowercase(),
        };
        h = (h ^ u64::from(b)).wrapping_mul(PRIME);
    }
    h & MASK
}

/// A stored hash without its reference bit.
#[must_use]
pub const fn strip(stored: u64) -> u64 {
    stored & MASK
}

/// Whether a stored hash is a reference (the asset is defined elsewhere).
#[must_use]
pub const fn is_ref(stored: u64) -> bool {
    stored & REF_BIT != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_found_in_the_zones() {
        // (Each was found stored as a u64 in a retail zone.)
        assert_eq!(name("jup_jp01_ar_golf36_mp"), 0x1d99_ae29_a03b_23d7);
        assert_eq!(name("iw9_ar_mike4_mp"), 0x66e5_6066_bcb9_809c);
        assert_eq!(name("vm_jup_jp01_ar_golf36_pgrip_ske_v3011"), 0x4264_7e35_2987_adb2);
        assert_eq!(name("mp_av_jup_jp01_ar_golf36_pgrip_ske_v3011_lod0_tr"), 0x4e2a_184e_3827_1b51);
        assert_eq!(name("mp_jup_rust"), 0x45ac_400e_a6d2_ee52);
        assert_eq!(name("JUP_WEAPON/JUP_JP01_AR_GOLF36"), 0x0d38_211c_17a1_4b64);
        assert_eq!(name("COOP_CRAFTING/DRONE_STRIKE"), 0x53dc_05c9_29ff_5c9e);
    }

    #[test]
    fn case_and_slashes_fold() {
        assert_eq!(name("MP_JUP_RUST"), name("mp_jup_rust"));
        assert_eq!(name("a\\b"), name("a/b"));
    }

    #[test]
    fn reference_bit() {
        let h = name("mp_jup_rust");
        assert!(!is_ref(h));
        assert!(is_ref(h | REF_BIT));
        assert_eq!(strip(h | REF_BIT), h);
    }
}
