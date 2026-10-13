use weapon_iw4::{HITLOC_COUNT, HITLOC_TORSO_MID, LOCATION_DAMAGE_IDENTITY, hitloc_slot, location_damage_is_valid};

const LOCHIT_MAGIC: &str = "LOCDMGTABLE";

/// Black Ops II's own `info/mp_lochit_dmgtable` (its common_mp zone, read
/// from the retail file): every bullet's damage is scaled by this and the
/// gun's own multipliers.
pub const BO2_LOCHIT_DMGTABLE: &str = "LOCDMGTABLE\\gun\\0.0\\head\\1.5\\helmet\\1.5\\left_arm_lower\\0.4\\left_arm_upper\\0.9\\left_foot\\0.4\\left_hand\\0.4\\left_leg_lower\\0.5\\left_leg_upper\\0.6\\neck\\1.5\\none\\1.0\\right_arm_lower\\0.4\\right_arm_upper\\0.9\\right_foot\\0.4\\right_hand\\0.4\\right_leg_lower\\0.5\\right_leg_upper\\0.6\\shield\\0.0\\torso_lower\\0.8\\torso_mid\\0.9\\torso_upper\\0.9";

/// Black Ops II's global location table, parsed.
pub fn bo2_lochit_table() -> [f32; HITLOC_COUNT] {
    parse_lochit_info_string(BO2_LOCHIT_DMGTABLE).unwrap_or(LOCATION_DAMAGE_IDENTITY)
}
const LOCHIT_RAWFILE: &str = "info/mp_lochit_dmgtable";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LochitTableError {
    MissingMagic,
    OddPair,
    BadNumber,
    UnknownKey,
    NotUtf8,
}

pub fn parse_lochit_info_string(text: &str) -> Result<[f32; HITLOC_COUNT], LochitTableError> {
    let text = text.trim();
    let rest = text
        .strip_prefix(LOCHIT_MAGIC)
        .ok_or(LochitTableError::MissingMagic)?;
    let mut table = LOCATION_DAMAGE_IDENTITY;
    let mut torso_mid = false;
    let mut parts = rest.split('\\').filter(|part| !part.is_empty());
    loop {
        let Some(key) = parts.next() else {
            break;
        };
        let Some(value) = parts.next() else {
            return Err(LochitTableError::OddPair);
        };
        let parsed: f32 = value.parse().map_err(|_| LochitTableError::BadNumber)?;
        let Some(slot) = hitloc_slot(key) else {
            return Err(LochitTableError::UnknownKey);
        };
        torso_mid |= slot == HITLOC_TORSO_MID;
        table[slot] = parsed;
    }
    // An IW game's table has no torso_mid: it is the upper torso there.
    if !torso_mid {
        table[HITLOC_TORSO_MID] = table[4];
    }
    if !location_damage_is_valid(&table) {
        return Err(LochitTableError::BadNumber);
    }
    Ok(table)
}

pub fn parse_lochit_rawfile(data: &[u8]) -> Result<[f32; HITLOC_COUNT], LochitTableError> {
    let text = core::str::from_utf8(data).map_err(|_| LochitTableError::NotUtf8)?;
    parse_lochit_info_string(text)
}

pub fn capture_lochit_table(
    name: &str,
    data: &[u8],
    zlib_compressed: bool,
) -> Option<[f32; HITLOC_COUNT]> {
    if name != LOCHIT_RAWFILE {
        return None;
    }
    let mut bytes = if zlib_compressed {
        asset_transport::inflate_zlib(data).ok()?
    } else {
        data.to_vec()
    };
    if bytes.last() == Some(&0) {
        bytes.pop();
    }
    parse_lochit_rawfile(&bytes).ok()
}

#[cfg(test)]
mod bo2_tests {
    use super::*;

    #[test]
    fn bo2_table_parses_into_its_own_order() {
        let t = bo2_lochit_table();
        let at = |n: &str| t[hitloc_slot(n).unwrap()];
        assert_eq!(at("head"), 1.5);
        assert_eq!(at("torso_upper"), 0.9);
        assert_eq!(at("torso_mid"), 0.9);
        assert_eq!(at("torso_lower"), 0.8);
        assert_eq!(at("left_leg_lower"), 0.5);
        assert_eq!(at("right_hand"), 0.4);
        assert_eq!(at("gun"), 0.0);
        assert_eq!(at("riotshield"), 0.0);
    }

    #[test]
    fn an_iw_table_gets_torso_mid_from_the_upper_torso() {
        let t = parse_lochit_info_string("LOCDMGTABLE\\torso_upper\\1.2\\torso_lower\\0.7\\shield\\0.0").unwrap();
        assert_eq!(t[HITLOC_TORSO_MID], 1.2);
        assert_eq!(t[weapon_iw4::HITLOC_RIOTSHIELD], 0.0);
    }
}
