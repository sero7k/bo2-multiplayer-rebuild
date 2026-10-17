use playerstate_iw4::{PlayerState, pm_flags};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdsFracContext {
    pub aim_down_sight: bool,

    pub ads_in_rate: f32,

    pub ads_out_rate: f32,

    pub rechamber_while_ads: bool,

    pub ads_fire_only: bool,

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

    // A knife swing keeps the sights up.
    let ws = ps.weaponstate_primary;

    let mut ads_requested = (ps.pm_flags & pm_flags::ADS_INTENT) != 0;
    if !context.rechamber_while_ads && ws == 7 {
        ads_requested = false;
    }
    if context.reload_lowers_ads && reload_holds_sights_down(ps, context) {
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

