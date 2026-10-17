//! Black Ops II's movement rules and the dvars that tune them.

use playerstate_iw4::{PlayerState, UserCmd, pm_flags};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bo2Feel {
    /// player_sprintStrafeSpeedScale: sideways input is scaled by this while
    /// sprinting.
    pub sprint_strafe_speed_scale: f32,

    /// jump_slowdownEnable: landing a jump slows you down (BO2: on in
    /// multiplayer, off in Zombies).
    pub jump_slowdown: bool,

    /// Falls higher than this hurt (bg_fallDamageMinHeight, 128 in both modes).
    pub fall_damage_min_height: f32,

    /// Falls this high or higher do full damage (bg_fallDamageMaxHeight, 300
    /// in both modes).
    pub fall_damage_max_height: f32,

    /// Zombies: no fall damage while this player's gravity is low, and PhD
    /// Flopper turns any fall damage into 1.
    pub zombies: bool,

    /// bg_gravity, for the Zombies low-gravity rule.
    pub gravity: i32,

    /// The fall perk's bit in perks[0] (specialty_fallheight): no fall
    /// damage, in both modes. 0 = none.
    pub fall_damage_perk: u32,

    /// PhD Flopper's bit in perks[0] (specialty_flakjacket): in Zombies any
    /// fall damage becomes 1, so the scripts can set off its blast. 0 = none.
    pub flak_jacket_perk: u32,

    /// Quickdraw's bit in perks[0] (specialty_fastads). 0 = none.
    pub fast_ads_perk: u32,

    /// perk_weapAdsMultiplier: with Quickdraw, aiming in and out takes this
    /// share of the gun's own time (0.5 until read from real BO2); sniper rifles
    /// are left alone.
    pub fast_ads_multiplier: f32,

    /// Stamin-Up's bit in perks[0] (specialty_longersprint). 0 = none.
    pub longer_sprint_perk: u32,

    /// perk_sprintMultiplier: Stamin-Up multiplies the sprint time by this
    /// (BO2 default 2, still capped at 16383 ms).
    pub sprint_multiplier: f32,

    /// Move-faster's bit in perks[0] (specialty_movefaster). 0 = none.
    pub move_faster_perk: u32,

    /// perk_speedMultiplier: move-faster multiplies the walk speed by this
    /// (BO2 default 1.07: 190 -> 203).
    pub speed_multiplier: f32,

    /// g_speed: the walk speed every frame starts from (190).
    pub g_speed: f32,

    /// dtp: dive to prone is allowed (BO2 default 1, in both modes).
    pub dive: bool,

    /// The gun in hand's footstep rhythm scale while sprinting
    /// (fSprintCycleScale), while sprinting crouched
    /// (fDuckedSprintCycleScale) and through a dive (fDtpCycleScale).
    pub sprint_cycle_scale: f32,
    pub ducked_sprint_cycle_scale: f32,
    pub dtp_cycle_scale: f32,

    /// Sprint in any direction at full speed. Not BO2 (it sprints forward
    /// only), so off unless the Controls menu's "Omni Movement" is on
    /// (off by default, an option for whoever wants it).
    pub omni: bool,

    /// mantle_weapon_height: a climb this low (by its climb table height)
    /// keeps the gun in hand; mantle_weapon_anim_height: one higher than
    /// this also plays the gun's climb camera animation. 0 lowers the gun
    /// for every climb.
    pub mantle_weapon_height: f32,
    pub mantle_weapon_anim_height: f32,
}

impl Bo2Feel {
    /// Black Ops II's multiplayer defaults (no dvar set by a script).
    pub const DEFAULTS: Self = Self {
        sprint_strafe_speed_scale: 0.667,
        jump_slowdown: true,
        fall_damage_min_height: 128.0,
        fall_damage_max_height: 300.0,
        zombies: false,
        gravity: 800,
        fall_damage_perk: PERK_FALLHEIGHT,
        flak_jacket_perk: PERK_FLAKJACKET,
        fast_ads_perk: PERK_FASTADS,
        fast_ads_multiplier: 0.5,
        longer_sprint_perk: PERK_LONGERSPRINT,
        sprint_multiplier: 2.0,
        move_faster_perk: PERK_MOVEFASTER,
        speed_multiplier: 1.07,
        g_speed: 190.0,
        dive: true,
        sprint_cycle_scale: 1.0,
        ducked_sprint_cycle_scale: 1.0,
        dtp_cycle_scale: 1.0,
        omni: false,
        mantle_weapon_height: 39.0,
        mantle_weapon_anim_height: 32.0,
    };
}

/// Sprint in any direction: the sprint rules see how far the stick (or the
/// keys) push in any direction as a push forward. Without it, BO2's client
/// sends no sprint key while moving back (CL_KeyMove), so letting go of back
/// with the sprint key still held starts a sprint again.
#[must_use]
pub fn omni_sprint_cmd(cmd: &UserCmd, feel: Bo2Feel) -> UserCmd {
    let mut out = *cmd;
    if feel.omni {
        let (f, r) = (f32::from(cmd.forwardmove), f32::from(cmd.rightmove));
        out.forwardmove = libm::sqrtf(f * f + r * r).min(127.0) as i8;
    } else if cmd.forwardmove < 0 {
        out.buttons &= !BUTTON_SPRINT;
    }
    out
}

/// Sprint in any direction at full speed: while sprinting, moving back or
/// sideways is not slowed.
#[must_use]
pub fn omni_cmd_scale(
    ps: &PlayerState,
    scale: crate::CmdScaleWalkContext,
    feel: Bo2Feel,
) -> crate::CmdScaleWalkContext {
    if feel.omni && ps.pm_flags & pm_flags::SPRINTING != 0 {
        crate::CmdScaleWalkContext {
            player_back_speed_scale: 1.0,
            player_strafe_speed_scale: 1.0,
            ..scale
        }
    } else {
        scale
    }
}

/// BO2 feel M14: a shellshock slows walking by the shock file's own number
/// (bg_shock_movement: explosion and pain 1, so not slowed; flashbang 0.8;
/// most others 0.4). `movement` is the file's switch; a host that puts the
/// number on the move-speed multiplier itself (bo2mp's `shellshock`)
/// turns it off, so the shock never slows twice.
#[must_use]
pub fn shellshock_walk_scale(movement: bool, movement_scale: f32) -> f32 {
    if !movement {
        1.0
    } else if movement_scale > 0.0 {
        movement_scale
    } else {
        1.0
    }
}

/// Fall perk (specialty_fallheight).
pub const PERK_FALLHEIGHT: u32 = 1 << 29;

/// Flak Jacket, Zombies' PhD Flopper (specialty_flakjacket).
pub const PERK_FLAKJACKET: u32 = 1 << 31;

/// Quickdraw (specialty_fastads).
pub const PERK_FASTADS: u32 = 1 << 30;

/// Stamin-Up, multiplayer's Extreme Conditioning (specialty_longersprint).
pub const PERK_LONGERSPRINT: u32 = 1 << 26;

/// Stalker (specialty_stalker): aiming does not slow the walk.
pub const PERK_STALKER: u32 = 1 << 11;

/// BO2 feel M10d (PM_UpdatePlayerWalkingFlag, run after the sprint update):
/// holding aim while the aim is on marks the player as walking, which slows
/// them to 0.4 and switches to the gun's aiming speed scale (cmd_scale_walk),
/// and quiets footsteps and bob. Not prone, not while reloading, not with
/// Stalker.
pub fn update_walking_flag(ps: &mut PlayerState, cmd: &UserCmd) {
    ps.pm_flags &= !pm_flags::WALKING;
    if ps.pm_type < playerstate_iw4::PM_TYPE_DEAD
        && cmd.buttons & crate::BUTTON_ADS != 0
        && ps.pm_flags & pm_flags::PRONE == 0
        && ps.pm_flags & pm_flags::ADS_INTENT != 0
        && !matches!(ps.weaponstate_primary, 0x8..=0xc)
        && ps.perks[0] & PERK_STALKER == 0
    {
        ps.pm_flags |= pm_flags::WALKING;
    }
}

/// The crouch-to-prone stage of the eye-height change takes 600 ms, and the
/// stance speed blends over the same time.
pub const PRONE_LERP_MS: i32 = 600;

/// Move-faster (specialty_movefaster): multiplayer's Lightweight speed and
/// the Turned zombies' perk.
pub const PERK_MOVEFASTER: u32 = 1 << 9;

/// BO2 feel M8b: every frame the walk speed starts again from g_speed; with
/// move-faster it is multiplied by perk_speedMultiplier and cut to a whole
/// number (190 x 1.07 = 203).
#[must_use]
pub fn player_speed(ps: &PlayerState, feel: Bo2Feel) -> i32 {
    let mut speed = feel.g_speed;
    if feel.move_faster_perk != 0 && ps.perks[0] & feel.move_faster_perk != 0 {
        speed *= feel.speed_multiplier;
    }
    speed as i32
}

/// In Zombies, Stamin-Up raises the gun's move speed to at least this while
/// sprinting (BO2's own number).
pub const STAMIN_UP_SPRINT_MOVE_SPEED_SCALE: f32 = 1.1;

fn stamin_up(ps: &PlayerState, feel: Bo2Feel) -> bool {
    feel.longer_sprint_perk != 0 && (ps.perks[0] & feel.longer_sprint_perk) != 0
}

/// BO2 feel M13: Stamin-Up multiplies the sprint time, capped at 16383 ms.
#[must_use]
pub fn stamin_up_sprint_time(ps: &PlayerState, max_ms: i32, feel: Bo2Feel) -> i32 {
    if !stamin_up(ps, feel) {
        return max_ms;
    }
    ((max_ms as f32 * feel.sprint_multiplier) as i32).min(0x3fff)
}

/// BO2 feel M13: in Zombies, Stamin-Up sprints with the gun's move speed at
/// least 1.1 (most guns are 1, so 10% faster). A gun with no move speed of
/// its own keeps its aiming speed.
#[must_use]
pub fn stamin_up_cmd_scale(
    ps: &PlayerState,
    scale: crate::CmdScaleWalkContext,
    feel: Bo2Feel,
) -> crate::CmdScaleWalkContext {
    if feel.zombies
        && stamin_up(ps, feel)
        && ps.pm_flags & pm_flags::SPRINTING != 0
        && scale.weapon_move_speed_scale > 0.0
    {
        crate::CmdScaleWalkContext {
            weapon_move_speed_scale: scale
                .weapon_move_speed_scale
                .max(STAMIN_UP_SPRINT_MOVE_SPEED_SCALE),
            ..scale
        }
    } else {
        scale
    }
}

/// BO2 feel G3h: Quickdraw speeds up aiming in and out, except on sniper
/// rifles. Returns the aim-in and aim-out rates (per ms).
#[must_use]
pub fn quickdraw_ads_rates(
    ps: &PlayerState,
    ads_in_rate: f32,
    ads_out_rate: f32,
    sniper: bool,
    feel: Bo2Feel,
) -> (f32, f32) {
    if sniper
        || feel.fast_ads_perk == 0
        || (ps.perks[0] & feel.fast_ads_perk) == 0
        || feel.fast_ads_multiplier <= 0.0
    {
        return (ads_in_rate, ads_out_rate);
    }
    (
        ads_in_rate / feel.fast_ads_multiplier,
        ads_out_rate / feel.fast_ads_multiplier,
    )
}

impl Default for Bo2Feel {
    fn default() -> Self {
        Self::DEFAULTS
    }
}

/// Pressing the sprint key again mid-sprint does nothing: the sprint rules
/// see the key as already held last frame, so a fresh press never counts.
#[must_use]
pub fn sprint_old_buttons(old_buttons: u32) -> u32 {
    old_buttons | BUTTON_SPRINT
}

const BUTTON_SPRINT: u32 = 0x2;

/// Extra time added to the sprint end when a sprint is ended by jumping.
pub const SPRINT_END_JUMP_PENALTY_MS: i32 = 800;

const BUTTON_JUMP: u32 = 0x400;

/// Jumping ends a sprint and moves the sprint end 800 ms later, so the next
/// sprint is shorter. Runs right after the sprint update; `before` is the
/// sprinting flag and sprint delay from just before it. A sprint that ran
/// out of time (it sets the sprint delay) does not count.
pub(crate) fn jump_out_of_sprint(
    ps: &mut PlayerState,
    cmd: &UserCmd,
    before: (u32, i32),
    change: crate::SprintResult,
) {
    if change == crate::SprintResult::Ended
        && before.0 != 0
        && ps.sprint_delay == before.1
        && (cmd.buttons & BUTTON_JUMP) != 0
    {
        ps.last_sprint_end = ps.last_sprint_end.wrapping_add(SPRINT_END_JUMP_PENALTY_MS);
    }
}

/// A landing counts as "landed higher" only this far above the take-off.
pub const JUMP_LAND_RAISE: f32 = 18.0;

/// Ground friction scale during the hard-landing stun.
pub const HARD_LANDING_FRICTION_SCALE: f32 = 0.3;

/// Walking acceleration on slick ground (the old rule had 1).
pub const SLICK_ACCEL: f32 = 2.0;

/// player_sliding_friction: slick ground still slows you, this much times
/// your speed each second.
pub const SLIDING_FRICTION: f32 = 1.5;

/// Head bob and footstep rhythm per stance: stand, prone, crouch, then the
/// same three running backwards. Columns: running, walking.
const BOB_FACTOR_TABLE: [[f32; 2]; 6] = [
    [0.335, 0.28],
    [0.25, 0.24],
    [0.34, 0.29],
    [0.36, 0.3],
    [0.25, 0.24],
    [0.34, 0.29],
];

/// player_moveThreshhold: slower than this counts as standing still.
pub const PLAYER_MOVE_THRESHHOLD: f32 = 2.0;

/// player_runbkThreshhold: standing up and slower than this walks.
const PLAYER_RUNBK_THRESHHOLD: f32 = 60.0;

/// player_sprintThreshhold: standing up, sprinting only counts this fast.
const PLAYER_SPRINT_THRESHHOLD: f32 = 185.0;

/// player_sprintCameraBob.
const PLAYER_SPRINT_CAMERA_BOB: f32 = 0.5;

/// Ladder rhythm: climbing speed against 0.75 x 127, walking 0.4 of that.
const LADDER_MAX_SPEED: f32 = 0.5 * 1.5 * 127.0;
const LADDER_BOB: f32 = 0.45;
const LADDER_WALK_BOB: f32 = 0.35;

/// Moves the head bob and footstep rhythm one frame on the ground. The caller
/// has already ruled out standing still, no input, the air and turrets.
pub(crate) fn bob_cycle(
    ps: &mut PlayerState,
    msec: i32,
    forwardmove: i8,
    rightmove: i8,
    xyspeed: f32,
    server_time: i32,
    scales: crate::CmdScaleWalkContext,
    feel: Bo2Feel,
) {
    let scales = stamin_up_cmd_scale(ps, scales, feel);
    // Downed (22) uses the prone row.
    let stance = match ps.view_height_target {
        22 | 11 => 1,
        40 => 2,
        _ => 0,
    };
    let backward = (ps.pm_flags & pm_flags::BACKWARDS_RUN) != 0;
    let row = stance + 3 * usize::from(backward);
    let mut walking = (ps.pm_flags & pm_flags::WALKING) != 0 || ps.leanf != 0.0;
    let mut sprinting = (ps.pm_flags & pm_flags::SPRINTING) != 0;
    if stance == 0 {
        walking |= xyspeed <= PLAYER_RUNBK_THRESHHOLD;
        sprinting &= xyspeed >= PLAYER_SPRINT_THRESHHOLD;
    }
    let max_speed = crate::get_bob_max_speed(
        ps,
        forwardmove,
        rightmove,
        walking,
        sprinting,
        server_time,
        scales,
    ) * ps.move_speed_scale_multiplier;
    if max_speed <= 0.0 {
        return;
    }
    let factor = if row == 0 && sprinting {
        PLAYER_SPRINT_CAMERA_BOB
    } else {
        BOB_FACTOR_TABLE[row][usize::from(walking)]
    };
    let bobmove = xyspeed / max_speed * factor;
    // The gun in hand speeds up or slows the rhythm while the sprint flag
    // is on (crouched: its own scale), whatever the speed.
    let sprint_flag = (ps.pm_flags & pm_flags::SPRINTING) != 0;
    let scale = if sprint_flag && ps.view_height_target == 40 {
        feel.ducked_sprint_cycle_scale
    } else if sprint_flag {
        feel.sprint_cycle_scale
    } else {
        1.0
    };
    ps.bob_cycle = step_cycle(ps.bob_cycle, msec, bobmove, scale);
}

/// Rhythm speed through a dive (dtpBobMove).
const DTP_BOB_MOVE: f32 = 0.24;

/// Through a whole dive, air and slide, the rhythm runs at its own pace
/// (times the gun's dive scale) and the normal step rhythm is skipped.
/// Returns true while diving.
pub(crate) fn dive_bob_cycle(ps: &mut PlayerState, msec: i32, feel: Bo2Feel) -> bool {
    if (ps.pm_flags & (crate::dive::PMF_DIVE | crate::dive::PMF_DIVE_SLIDE)) == 0 {
        return false;
    }
    ps.bob_cycle = step_cycle(ps.bob_cycle, msec, DTP_BOB_MOVE, feel.dtp_cycle_scale);
    true
}

/// Ladder rhythm: follows the climbing speed (down runs it backwards).
pub(crate) fn ladder_bob_cycle(ps: &mut PlayerState, msec: i32) {
    let bobmove = if (ps.pm_flags & pm_flags::WALKING) == 0 && ps.leanf == 0.0 {
        ps.velocity[2] / LADDER_MAX_SPEED * LADDER_BOB
    } else {
        ps.velocity[2] / (LADDER_MAX_SPEED * 0.4) * LADDER_WALK_BOB
    };
    ps.bob_cycle = step_cycle(ps.bob_cycle, msec, bobmove, 1.0);
}

/// The rhythm counter steps by whole units, rounding down.
fn step_cycle(old: i32, msec: i32, bobmove: f32, scale: f32) -> i32 {
    let old = old as u8;
    i32::from((f32::from(old) + msec as f32 * bobmove * scale) as i32 as u8)
}

/// EV_RESET_ADS: the gun drops out of its aim.
const EV_RESET_ADS: i32 = 0x11;

/// Black Ops II's crawl and aim (PmoveSingle, before the aim update, so it
/// reads last frame's aim). Lying down (or downed) and aiming holds the
/// player still, unless a scope is fully raised. Pressing a move key harder
/// than last frame drops the aim and crawls, but not while firing, meleeing,
/// throwing or starting a sprint; the hold stays until the keys ease off with
/// the gun at rest. Also sets the run-backwards flag from the keys.
pub(crate) fn prone_aim(
    ps: &mut PlayerState,
    cmd: &mut UserCmd,
    old_move: [i8; 2],
    scope_raised: bool,
) {
    if ps.pm_flags & pm_flags::PRONE == 0 || scope_raised {
        ps.pm_flags &= !pm_flags::PRONEMOVE_OVERRIDDEN;
    } else if !pushed_harder(cmd.forwardmove, old_move[0])
        && !pushed_harder(cmd.rightmove, old_move[1])
    {
        // Ready, raising, dropping or reloading.
        if ps.pm_flags & pm_flags::ADS_INTENT == 0
            && matches!(ps.weaponstate_primary, 0..=5 | 8)
        {
            ps.pm_flags &= !pm_flags::PRONEMOVE_OVERRIDDEN;
        }
    } else if !crate::dive::active(ps) && prone_move_interrupts_weapon(ps.weaponstate_primary) {
        ps.pm_flags &= !(pm_flags::PRONEMOVE_OVERRIDDEN | pm_flags::ADS_INTENT);
        crate::add_event(ps, EV_RESET_ADS);
    }

    let lying = matches!(ps.view_height_target, 11 | 22);
    let aiming = ps.pm_flags & pm_flags::ADS_INTENT != 0 && !scope_raised;
    let diving = crate::dive::active(ps);
    if lying && aiming && !diving {
        cmd.forwardmove = 0;
        cmd.rightmove = 0;
    }
    if cmd.forwardmove < 0 {
        ps.pm_flags |= pm_flags::BACKWARDS_RUN;
    } else if cmd.forwardmove > 0 || cmd.rightmove != 0 {
        ps.pm_flags &= !pm_flags::BACKWARDS_RUN;
    }
    if lying && ps.pm_flags & pm_flags::PRONEMOVE_OVERRIDDEN != 0 && !diving {
        cmd.forwardmove = 0;
        cmd.rightmove = 0;
    }
}

fn pushed_harder(now: i8, before: i8) -> bool {
    now != before && i16::from(now).abs() > i16::from(before).abs()
}

/// BO2's PM_InteruptWeaponWithProneMove: a crawl cannot cut into firing, a
/// melee swing, a throw, a detonation, the sprint raise or taking night
/// vision off (our weapon states).
fn prone_move_interrupts_weapon(weaponstate: i32) -> bool {
    !matches!(weaponstate, 6 | 0xE | 0xF | 0x10..=0x17 | 0x1E)
}

