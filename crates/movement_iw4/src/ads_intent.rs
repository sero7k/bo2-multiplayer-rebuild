use crate::add_predictable_event;
use playerstate_iw4::{PlayerState, UserCmd, pm_flags};

const BUTTON_SPRINT: u32 = 0x2;

pub const BUTTON_ADS: u32 = 0x800;

const BUTTON_SCOPE_HOLD: u32 = 0x2000;

const EV_RESET_ADS: i32 = 0x11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdsIntentContext {
    pub ads_allowed: bool,

    pub weapon_def_scope: bool,

    pub sprint_hold_ads: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdsIntentResult {
    pub ads_anim_enabled: bool,

    pub exit_ads_event: bool,
}

/// BO2's BG_UsingSniperScope: a scope overlay, fully raised.
pub(crate) fn sniper_scope_raised(ps: &PlayerState, context: AdsIntentContext) -> bool {
    context.weapon_def_scope && ps.f_weapon_pos_frac >= 1.0
}

/// A scope keeps its aim while crawling only once fully raised.
fn scoped_weapon_raised(ps: &PlayerState, context: AdsIntentContext) -> bool {
    context.weapon_def_scope && ps.f_weapon_pos_frac >= 1.0
}

pub fn update_ads_intent(
    ps: &mut PlayerState,
    cmd: &UserCmd,
    old_buttons: u32,
    context: AdsIntentContext,
) -> AdsIntentResult {
    ps.pm_flags &= !pm_flags::ADS_INTENT;

    let mut ads_allowed = context.ads_allowed;
    let mut exit_ads_event = false;

    // Aiming beats a held sprint button (aim from a sprint with shift still
    // down); only sprint without aim drops ADS.
    if (cmd.buttons & BUTTON_SPRINT) != 0
        && (cmd.buttons & BUTTON_ADS) == 0
        && (!context.weapon_def_scope || (cmd.buttons & BUTTON_SCOPE_HOLD) == 0)
        && (!context.sprint_hold_ads || ps.f_weapon_pos_frac == 0.0)
    {
        add_predictable_event(ps, EV_RESET_ADS, 0);
        exit_ads_event = true;
        ps.pm_flags &= !pm_flags::ADS_INTENT;
        ads_allowed = false;
    }

    if (cmd.buttons & BUTTON_ADS) != 0 && ads_allowed {
        if (ps.pm_flags & pm_flags::PRONE) != 0 && !scoped_weapon_raised(ps, context) {
            let moving = cmd.forwardmove != 0 || cmd.rightmove != 0;
            if (old_buttons & BUTTON_ADS) == 0 || !moving {
                ps.pm_flags |= pm_flags::ADS_INTENT | pm_flags::PRONEMOVE_OVERRIDDEN;
            }
        } else {
            ps.pm_flags |= pm_flags::ADS_INTENT;
        }
    }

    AdsIntentResult {
        ads_anim_enabled: (ps.pm_flags & pm_flags::ADS_INTENT) != 0,
        exit_ads_event,
    }
}

