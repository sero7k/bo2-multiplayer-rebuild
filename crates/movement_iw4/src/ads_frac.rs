use playerstate_iw4::{PlayerState, pm_flags};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdsFracContext {
    pub aim_down_sight: bool,

    pub ads_in_rate: f32,

    pub ads_out_rate: f32,

    pub rechamber_while_ads: bool,

    pub ads_fire_only: bool,

    /// BO2's lerp: a knife swing keeps the sights up, and a reload lowers
    /// them on guns that cannot aim while reloading.
    pub bo2: bool,

    /// BO2: a reload lowers the sights (!bReloadWhileAds).
    pub reload_lowers_ads: bool,

    /// BO2 iPositionReloadTransTime: the sights come back up when this much
    /// of the reload is left.
    pub position_reload_trans_ms: i32,

    pub segmented_reload: bool,
}

impl Default for AdsFracContext {
    fn default() -> Self {
        Self {
            aim_down_sight: true,

            ads_in_rate: 1.0 / 200.0,
            ads_out_rate: 1.0 / 200.0,
            rechamber_while_ads: true,
            ads_fire_only: false,
            bo2: false,
            reload_lowers_ads: false,
            position_reload_trans_ms: 0,
            segmented_reload: false,
        }
    }
}

pub fn update_ads_frac(ps: &mut PlayerState, msec: i32, context: AdsFracContext) {
    if !context.aim_down_sight {
        ps.f_weapon_pos_frac = 0.0;
        ps.ads_delay_time = 0;
        return;
    }

    let ws = ps.weaponstate_primary;
    if !context.bo2 && matches!(ws, 0xd | 0xe | 0xf) {
        ps.f_weapon_pos_frac = 0.0;
        ps.ads_delay_time = 0;
        return;
    }

    let mut ads_requested = (ps.pm_flags & pm_flags::ADS_INTENT) != 0;
    if !context.rechamber_while_ads && ws == 7 {
        ads_requested = false;
    }
    if context.bo2 && context.reload_lowers_ads && reload_holds_sights_down(ps, context) {
        ads_requested = false;
    }

    if context.ads_fire_only
        && ((ps.weapon_delay != 0 && ws == 6)
            || (ps.weapon_delay_secondary != 0 && ps.weaponstate_secondary == 6))
    {
        ads_requested = true;
    }
    let dt = msec.max(0) as f32;

    if ads_requested {
        if ps.f_weapon_pos_frac >= 1.0 {
            ps.f_weapon_pos_frac = 1.0;
            ps.ads_delay_time = 0;
            return;
        }
        let rate = context.ads_in_rate;
        if rate <= 0.0 {
            ps.f_weapon_pos_frac = 1.0;
        } else {
            ps.f_weapon_pos_frac = (ps.f_weapon_pos_frac + rate * dt).clamp(0.0, 1.0);
        }
        ps.ads_delay_time = 0;
    } else {
        if ps.f_weapon_pos_frac <= 0.0 {
            ps.f_weapon_pos_frac = 0.0;
            ps.ads_delay_time = 0;
            return;
        }

        let rate = context.ads_out_rate;
        if rate <= 0.0 {
            ps.f_weapon_pos_frac = 0.0;
        } else {
            ps.f_weapon_pos_frac = (ps.f_weapon_pos_frac - rate * dt).clamp(0.0, 1.0);
        }
        ps.ads_delay_time = 0;
    }
}

/// BO2: the sights stay down through the reload and come back up once only
/// iPositionReloadTransTime of it is left. A segmented reload keeps them down
/// for the start and every shell, and lets them up near the end.
fn reload_holds_sights_down(ps: &PlayerState, context: AdsFracContext) -> bool {
    let early = ps.weapon_time > context.position_reload_trans_ms;
    match ps.weaponstate_primary {
        0x8 => context.segmented_reload || early,
        0x9 | 0xa | 0xb => context.segmented_reload,
        0xc => context.segmented_reload && early,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aiming(ws: i32, weapon_time: i32) -> PlayerState {
        let mut ps = PlayerState::ZERO;
        ps.pm_flags |= pm_flags::ADS_INTENT;
        ps.f_weapon_pos_frac = 1.0;
        ps.weaponstate_primary = ws;
        ps.weapon_time = weapon_time;
        ps
    }

    fn bo2(reload_lowers_ads: bool, segmented_reload: bool) -> AdsFracContext {
        AdsFracContext {
            bo2: true,
            reload_lowers_ads,
            position_reload_trans_ms: 300,
            segmented_reload,
            ..AdsFracContext::default()
        }
    }

    #[test]
    fn bo2_reload_lowers_the_sights_then_raises_them_near_the_end() {
        let mut ps = aiming(0x8, 1500);
        update_ads_frac(&mut ps, 50, bo2(true, false));
        assert!(ps.f_weapon_pos_frac < 1.0, "early in the reload the sights come down");

        let mut ps = aiming(0x8, 300);
        ps.f_weapon_pos_frac = 0.5;
        update_ads_frac(&mut ps, 50, bo2(true, false));
        assert!(ps.f_weapon_pos_frac > 0.5, "with the last bit left they come back up");
    }

    #[test]
    fn bo2_gun_that_aims_while_reloading_keeps_the_sights() {
        let mut ps = aiming(0x8, 1500);
        update_ads_frac(&mut ps, 50, bo2(false, false));
        assert_eq!(ps.f_weapon_pos_frac, 1.0);
    }

    #[test]
    fn bo2_segmented_reload_keeps_sights_down_until_the_end() {
        for ws in [0x8, 0x9, 0xa, 0xb] {
            let mut ps = aiming(ws, 100);
            update_ads_frac(&mut ps, 50, bo2(true, true));
            assert!(ps.f_weapon_pos_frac < 1.0, "state {ws:#x}");
        }
        let mut ps = aiming(0xc, 600);
        update_ads_frac(&mut ps, 50, bo2(true, true));
        assert!(ps.f_weapon_pos_frac < 1.0, "reload end, early");
        let mut ps = aiming(0xc, 200);
        update_ads_frac(&mut ps, 50, bo2(true, true));
        assert_eq!(ps.f_weapon_pos_frac, 1.0, "reload end, last bit");
    }

    #[test]
    fn bo2_knife_swing_keeps_the_sights_up() {
        let mut ps = aiming(0xe, 200);
        update_ads_frac(&mut ps, 50, bo2(false, false));
        assert_eq!(ps.f_weapon_pos_frac, 1.0);

        let mut ps = aiming(0xe, 200);
        update_ads_frac(&mut ps, 50, AdsFracContext::default());
        assert_eq!(ps.f_weapon_pos_frac, 0.0, "MW2 drops them");
    }

    #[test]
    fn mw2_reload_keeps_the_sights() {
        let mut ps = aiming(0x8, 1500);
        let context = AdsFracContext { reload_lowers_ads: true, ..AdsFracContext::default() };
        update_ads_frac(&mut ps, 50, context);
        assert_eq!(ps.f_weapon_pos_frac, 1.0);
    }
}
