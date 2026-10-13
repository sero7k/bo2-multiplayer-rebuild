use crate::ammo::{AMMOCLIP_TABLE_BYTES, get_clip_for_hand};
use playerstate_iw4::PlayerState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdsAllowWeaponFacts {
    pub aim_down_sight: bool,

    pub no_ads_when_mag_empty: bool,

    pub clip_index: i32,

    /// bo2mp: Hybrid Optic: the sight swap keeps the player aimed.
    pub optic_switch: bool,
}

pub const OTHER_FLAG_PLAYER: u32 = playerstate_iw4::other_flags::PLAYER;

pub const WEAP_FLAG_NO_ADS: u32 = playerstate_iw4::weap_flags::NO_ADS;

pub fn is_ads_allowed(
    ps: &PlayerState,
    weap: &AdsAllowWeaponFacts,
    ammoclip: &[u8; AMMOCLIP_TABLE_BYTES],
    clip_hand: u8,
) -> bool {
    ads_allowed(ps, weap, ammoclip, clip_hand, false)
}

/// BO2's PM_IsAdsAllowed: a knife swing keeps the sights up (IW4 dropped
/// them), and climbing over a ledge drops them (IW4 let you aim).
pub fn is_ads_allowed_bo2(
    ps: &PlayerState,
    weap: &AdsAllowWeaponFacts,
    ammoclip: &[u8; AMMOCLIP_TABLE_BYTES],
    clip_hand: u8,
) -> bool {
    ads_allowed(ps, weap, ammoclip, clip_hand, true)
}

fn ads_allowed(
    ps: &PlayerState,
    weap: &AdsAllowWeaponFacts,
    ammoclip: &[u8; AMMOCLIP_TABLE_BYTES],
    clip_hand: u8,
    bo2: bool,
) -> bool {
    if ps.last_weapon_hand == 1 && (ps.e_flags & 0xc00) == 0 {
        return false;
    }

    match ps.pm_type {
        2 | 3 | 5 | 6 | 8 | 9 => return false,
        _ => {}
    }

    if (ps.other_flags & playerstate_iw4::other_flags::PLAYER) == 0 {
        return false;
    }
    if !weap.aim_down_sight {
        return false;
    }

    let ws = ps.weaponstate_primary;

    if (0x10..=0x15).contains(&ws) {
        return false;
    }
    match ws {
        // bo2mp: the Hybrid Optic's swap (alternate raise/drop) stays in ADS.
        0x2 | 0x5 if weap.optic_switch => {}
        0x1 | 0x2 | 0x3 | 0x4 | 0x5 => return false,
        0xd | 0xe | 0xf if !bo2 => return false,
        0x1d | 0x1e => return false,
        _ => {}
    }

    if bo2 && (ps.pm_flags & playerstate_iw4::pm_flags::MANTLE) != 0 {
        return false;
    }

    if (ps.weap_flags & playerstate_iw4::weap_flags::NO_ADS) != 0 {
        return false;
    }

    if weap.no_ads_when_mag_empty {
        let clip = get_clip_for_hand(ammoclip, weap.clip_index, clip_hand);
        if clip == 0 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aiming_player() -> PlayerState {
        let mut ps = PlayerState::ZERO;
        ps.other_flags = OTHER_FLAG_PLAYER;
        ps
    }

    fn rifle() -> AdsAllowWeaponFacts {
        AdsAllowWeaponFacts {
            aim_down_sight: true,
            ..AdsAllowWeaponFacts::default()
        }
    }

    /// BO2's PM_IsAdsAllowed has no knife states in its list: the sights
    /// stay up through a knife swing. MW2 lowered them.
    #[test]
    fn bo2_keeps_aim_through_a_knife_swing() {
        let clips = [0u8; AMMOCLIP_TABLE_BYTES];
        let mut ps = aiming_player();
        for state in [0xd, 0xe, 0xf] {
            ps.weaponstate_primary = state;
            assert!(is_ads_allowed_bo2(&ps, &rifle(), &clips, 0));
            assert!(!is_ads_allowed(&ps, &rifle(), &clips, 0));
        }
    }

    /// BO2 blocks aiming while climbing over a ledge; MW2 did not.
    #[test]
    fn bo2_blocks_aim_while_mantling() {
        let clips = [0u8; AMMOCLIP_TABLE_BYTES];
        let mut ps = aiming_player();
        ps.pm_flags |= playerstate_iw4::pm_flags::MANTLE;
        assert!(!is_ads_allowed_bo2(&ps, &rifle(), &clips, 0));
        assert!(is_ads_allowed(&ps, &rifle(), &clips, 0));
        ps.pm_flags = 0;
        assert!(is_ads_allowed_bo2(&ps, &rifle(), &clips, 0));
    }
}
