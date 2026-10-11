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

    /// Black Ops II's rules (PM_UpdateAimDownSightFlag): a scope keeps its
    /// aim while crawling only once fully raised (IW4: as soon as it starts
    /// to raise), and prone has no settled exception (IW4's dive flag; that
    /// bit is last stand here).
    pub bo2: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdsIntentResult {
    pub ads_anim_enabled: bool,

    pub exit_ads_event: bool,
}

fn prone_stance_settled(ps: &PlayerState) -> bool {
    (ps.pm_flags & 0x800) == 0 && (ps.pm_flags & 0x400_000) != 0 && ps.pm_time == 0
}

/// BO2's BG_UsingSniperScope: a scope overlay, fully raised.
pub(crate) fn sniper_scope_raised(ps: &PlayerState, context: AdsIntentContext) -> bool {
    context.weapon_def_scope && ps.f_weapon_pos_frac >= 1.0
}

fn scoped_weapon_raised(ps: &PlayerState, context: AdsIntentContext) -> bool {
    context.weapon_def_scope
        && if context.bo2 {
            ps.f_weapon_pos_frac >= 1.0
        } else {
            ps.f_weapon_pos_frac > 0.0
        }
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

    if (cmd.buttons & BUTTON_SPRINT) != 0
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
            let settled = !context.bo2 && prone_stance_settled(ps);
            if (old_buttons & BUTTON_ADS) != 0 && moving && !settled {
            } else {
                ps.pm_flags |= pm_flags::ADS_INTENT;
                if !settled {
                    ps.pm_flags |= pm_flags::PRONEMOVE_OVERRIDDEN;
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// BO2's PM_UpdateAimDownSightFlag: crawling while holding aim keeps a
    /// scope up only once it is fully raised (MW2: once it starts raising).
    #[test]
    fn bo2_crawl_keeps_a_scope_only_when_fully_raised() {
        let cmd = UserCmd {
            forwardmove: 127,
            buttons: BUTTON_ADS,
            ..UserCmd::default()
        };
        let context = |bo2| AdsIntentContext {
            ads_allowed: true,
            weapon_def_scope: true,
            sprint_hold_ads: false,
            bo2,
        };
        for (frac, bo2, aims) in [
            (0.5, true, false),
            (0.5, false, true),
            (1.0, true, true),
            (1.0, false, true),
        ] {
            let mut ps = PlayerState::ZERO;
            ps.pm_flags = pm_flags::PRONE;
            ps.f_weapon_pos_frac = frac;
            let result = update_ads_intent(&mut ps, &cmd, BUTTON_ADS, context(bo2));
            assert_eq!(result.ads_anim_enabled, aims, "frac {frac} bo2 {bo2}");
        }
    }
}
