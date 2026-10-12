//! bo2mp: Call of Duty: Ghosts weapons into the weapon catalog, as guest
//! guns in a Black Ops II match.
//!
//! `Iw6WeaponView` carries a weapon's `WeaponCompleteDef` and `WeaponDef`
//! as bytes plus every name they point at (read by `fastfile_iw6`'s walk);
//! this reads the fields the engine's weapon rules, viewmodel and sounds use,
//! by their Ghosts offsets. Ghosts' enums follow IW4's numbering (class,
//! type, fire type) and its animation slots are mapped to the catalog's.

use fastfile_iw6::layout::{WeaponCompleteDef as c, WeaponDef as d};

use super::*;

/// One Ghosts weapon as the walk read it.
#[derive(Clone, Debug, Default)]
pub struct Iw6WeaponView {
    /// `iw6_ak12_mp`.
    pub name: String,
    pub display_name: String,
    /// `WeaponCompleteDef` bytes.
    pub complete: Vec<u8>,
    /// `WeaponDef` bytes (empty when the walk did not reach it).
    pub def: Vec<u8>,
    pub gun_model: Option<String>,
    pub world_model: Option<String>,
    /// The 124 animation slots (`weapAnimFiles_t`), empty for none.
    pub xanims: Vec<String>,
    pub hide_tags: Vec<String>,
    pub alt_weapon_name: String,
    /// (`WeaponDef` field offset, alias): every sound field.
    pub sounds: Vec<(usize, String)>,
    /// (`WeaponDef` field offset, effect).
    pub fx: Vec<(usize, String)>,
    /// (`WeaponDef` field offset, model).
    pub models: Vec<(usize, String)>,
    /// (`WeaponDef` field offset, material); `WeaponCompleteDef` fields as
    /// `usize::MAX - offset`.
    pub materials: Vec<(usize, String)>,
    /// The notetrack sound map (notetrack, alias).
    pub notetrack_sounds: Vec<(String, String)>,
    /// `locationDamageMultipliers`: one per hit location.
    pub location_damage: Option<Vec<f32>>,
    pub tracer: Option<String>,
}

impl Iw6WeaponView {
    fn rd<const N: usize>(b: &[u8], at: usize) -> Option<[u8; N]> {
        b.get(at..at + N)?.try_into().ok()
    }
    fn def_i32(&self, off: usize) -> i32 {
        Self::rd::<4>(&self.def, off).map_or(0, i32::from_le_bytes)
    }
    fn def_f32(&self, off: usize) -> f32 {
        Self::rd::<4>(&self.def, off).map_or(0.0, f32::from_le_bytes)
    }
    fn def_u8(&self, off: usize) -> u8 {
        self.def.get(off).copied().unwrap_or(0)
    }
    fn def_vec3(&self, off: usize) -> [f32; 3] {
        [self.def_f32(off), self.def_f32(off + 4), self.def_f32(off + 8)]
    }
    fn complete_i32(&self, off: usize) -> i32 {
        Self::rd::<4>(&self.complete, off).map_or(0, i32::from_le_bytes)
    }
    fn complete_f32(&self, off: usize) -> f32 {
        Self::rd::<4>(&self.complete, off).map_or(0.0, f32::from_le_bytes)
    }
    fn complete_u8(&self, off: usize) -> u8 {
        self.complete.get(off).copied().unwrap_or(0)
    }
    fn sound(&self, off: usize) -> Option<String> {
        self.sounds.iter().find(|(o, _)| *o == off).map(|(_, n)| n.clone()).filter(|n| !n.is_empty())
    }
    fn effect(&self, off: usize) -> Option<String> {
        self.fx.iter().find(|(o, _)| *o == off).map(|(_, n)| n.clone()).filter(|n| !n.is_empty())
    }
    fn model(&self, off: usize) -> Option<String> {
        self.models.iter().find(|(o, _)| *o == off).map(|(_, n)| n.clone()).filter(|n| !n.is_empty())
    }
    fn material(&self, off: usize) -> Option<String> {
        self.materials.iter().find(|(o, _)| *o == off).map(|(_, n)| n.clone()).filter(|n| !n.is_empty())
    }

    /// The catalog's name for it: `iw6_ak12_mp` -> `ak12` (the guest name
    /// is then `gh_ak12_mp`).
    pub fn catalog_name(&self) -> String {
        let n = self.name.to_ascii_lowercase();
        let n = n.strip_suffix("_mp").unwrap_or(&n);
        n.strip_prefix("iw6_").unwrap_or(n).to_owned()
    }
}

/// Ghosts `weapAnimFiles_t` slot -> the catalog's slot (IW4 numbering plus
/// the quick-reload extras).
const IW6_ANIM_SLOTS: [(usize, usize); 36] = [
    (0x01, weap_anim::IDLE),
    (0x02, weap_anim::EMPTY_IDLE),
    (0x03, weap_anim::FIRE),
    (0x04, weap_anim::HOLD_FIRE),
    (0x05, weap_anim::LASTSHOT),
    (0x06, weap_anim::RECHAMBER),
    // (Ghosts splits the melee into swipe, hit and miss; the swipe is the
    // swing every melee plays.)
    (0x09, weap_anim::MELEE),
    (0x10, weap_anim::RELOAD),
    (0x11, weap_anim::RELOAD_EMPTY),
    (0x12, weap_anim::RELOAD_START),
    (0x13, weap_anim::RELOAD_END),
    (0x14, weap_anim_extra::RELOAD_QUICK),
    (0x15, weap_anim_extra::RELOAD_QUICK_EMPTY),
    (0x18, weap_anim::RAISE),
    (0x19, weap_anim::FIRST_RAISE),
    (0x1A, weap_anim::BREACH_RAISE),
    (0x1B, weap_anim::DROP),
    (0x1C, weap_anim::ALT_RAISE),
    (0x1D, weap_anim::ALT_DROP),
    (0x22, weap_anim::QUICK_RAISE),
    (0x23, weap_anim::QUICK_DROP),
    (0x24, weap_anim::EMPTY_RAISE),
    (0x25, weap_anim::EMPTY_DROP),
    (0x26, weap_anim::SPRINT_IN),
    (0x28, weap_anim::SPRINT_LOOP),
    (0x29, weap_anim::SPRINT_OUT),
    (0x2A, weap_anim::STUNNED_START),
    (0x2B, weap_anim::STUNNED_LOOP),
    (0x2C, weap_anim::STUNNED_END),
    (0x2D, weap_anim::DETONATE),
    (0x2E, weap_anim::NIGHTVISION_WEAR),
    (0x2F, weap_anim::NIGHTVISION_REMOVE),
    (0x30, weap_anim::ADS_FIRE),
    (0x31, weap_anim::ADS_LASTSHOT),
    (0x32, weap_anim::ADS_RECHAMBER),
    (0x5A, weap_anim::ADS_UP),
];

fn remap_iw6_sz_xanims(iw6: &[String]) -> [Option<String>; WEAPON_ANIM_SLOTS] {
    let mut out = [const { None }; WEAPON_ANIM_SLOTS];
    for (src, dst) in IW6_ANIM_SLOTS {
        if let Some(name) = iw6.get(src).filter(|n| !n.is_empty()) {
            out[dst] = Some(name.to_ascii_lowercase());
        }
    }
    if let Some(name) = iw6.get(0x5B).filter(|n| !n.is_empty()) {
        out[weap_anim::ADS_DOWN] = Some(name.to_ascii_lowercase());
    }
    out
}

/// Ghosts `weapType_t`: bullet, grenade, projectile, riot shield (as IW4);
/// the rest are not fired like guns.
fn remap_iw6_weap_type(raw: i32) -> i32 {
    match raw {
        0..=3 => raw,
        _ => 0,
    }
}

/// Ghosts `weapFireType_t`: full auto, single, bursts 2..4 as IW4; the
/// double barrel and beyond fire single.
fn remap_iw6_fire_type(raw: i32) -> i32 {
    match raw {
        0..=4 => raw,
        _ => 1,
    }
}

fn iw6_body_facts(w: &Iw6WeaponView) -> WeaponBodyFacts {
    let ads_in = w.complete_i32(c::iAdsTransInTime);
    let ads_out = w.complete_i32(c::iAdsTransOutTime);
    let mut f = WeaponBodyFacts {
        body_resolved: !w.def.is_empty(),
        clip_size: w.complete_i32(c::iClipSize),
        fire_time_ms: w.complete_i32(c::iFireTime),
        impact_type: w.complete_i32(c::impactType),
        alternate_raise_time_ms: w.complete_i32(c::iAltRaiseTime),
        ads_zoom_fov: w.complete_f32(c::fAdsZoomFov),
        ads_in_rate: if ads_in > 0 { 1.0 / ads_in as f32 } else { 0.0 },
        ads_out_rate: if ads_out > 0 { 1.0 / ads_out as f32 } else { 0.0 },
        penetrate_multiplier: w.complete_f32(c::penetrateMultiplier),
        motion_tracker: w.complete_u8(c::motionTracker) != 0,
        select_requires_ammo: Some(false),
        ..WeaponBodyFacts::default()
    };
    let dof = [w.complete_f32(c::adsDofStart), w.complete_f32(c::adsDofEnd)];
    f.ads_dof = (dof[1] > 0.0).then_some(dof);
    f.kick.f_ads_view_kick_center_speed = w.complete_f32(c::fAdsViewKickCenterSpeed);
    f.kick.f_hip_view_kick_center_speed = w.complete_f32(c::fHipViewKickCenterSpeed);
    if w.def.is_empty() {
        return f;
    }
    let t = |off: usize| w.def_i32(off);
    f.raise_time_ms = t(d::stateTimers__iRaiseTime);
    f.drop_time_ms = t(d::stateTimers__iDropTime);
    f.alternate_drop_time_ms = t(d::stateTimers__iAltDropTime);
    f.fire_delay_ms = t(d::stateTimers__iFireDelay);
    f.hold_fire_time_ms = t(d::stateTimers__iHoldFireTime);
    f.melee_time_ms = t(d::stateTimers__iMeleeTime);
    f.melee_delay_ms = t(d::stateTimers__iMeleeDelay);
    f.melee_charge_time_ms = t(d::stateTimers__meleeChargeTime);
    f.melee_charge_delay_ms = t(d::stateTimers__meleeChargeDelay);
    f.rechamber_time_ms = t(d::stateTimers__iRechamberTime);
    f.rechamber_bolt_time_ms = t(d::stateTimers__iRechamberBoltTime);
    f.reload_time_ms = t(d::stateTimers__iReloadTime);
    f.reload_empty_time_ms = t(d::stateTimers__iReloadEmptyTime);
    f.reload_show_rocket_time_ms = t(d::stateTimers__reloadShowRocketTime);
    f.reload_add_time_ms = t(d::stateTimers__iReloadAddTime);
    f.reload_empty_add_time_ms = t(d::stateTimers__iReloadEmptyAddTime);
    f.reload_start_time_ms = t(d::stateTimers__iReloadStartTime);
    f.reload_start_add_time_ms = t(d::stateTimers__iReloadStartAddTime);
    f.reload_end_time_ms = t(d::stateTimers__iReloadEndTime);
    f.quick_raise_time_ms = t(d::stateTimers__quickRaiseTime);
    f.quick_drop_time_ms = t(d::stateTimers__quickDropTime);
    f.sprint_raise_time_ms = t(d::stateTimers__sprintInTime);
    f.sprint_loop_time_ms = t(d::stateTimers__sprintLoopTime);
    f.sprint_drop_time_ms = t(d::stateTimers__sprintOutTime);
    f.night_vision_wear_time = t(d::stateTimers__nightVisionWearTime);
    f.fuse_time_ms = t(d::stateTimers__fuseTime);
    f.detonate_delay_ms = t(d::stateTimers__iDetonateDelay);
    f.detonate_time_ms = t(d::stateTimers__iDetonateTime);
    f.weap_type = remap_iw6_weap_type(w.def_i32(d::weapType));
    f.player_anim_type = w.def_i32(d::playerAnimType);
    f.weap_class = match w.def_i32(d::weapClass) {
        k @ 0..=11 => k,
        _ => 0,
    };
    f.offhand_class = w.def_i32(d::offhandClass);
    f.inventory_type = match w.def_i32(d::inventoryType) {
        k @ 0..=3 => k,
        _ => 0,
    };
    f.fire_type = remap_iw6_fire_type(w.def_i32(d::fireType));
    f.shots_per_fire = w.def_i32(d::shotCount);
    f.ammo_counter_clip = w.def_i32(d::ammoCounterClip);
    f.low_ammo_warning_threshold = w.def_f32(d::lowAmmoWarningThreshold);
    apply_leftover_hip_spread(&mut f, leftover_hip_spread_block(|off| w.def_f32(off), d::fHipSpreadStandMin));
    f.i_reticle_side_size = w.def_i32(d::iReticleSideSize);
    f.i_reticle_min_ofs = w.def_i32(d::iReticleMinOfs);
    f.hip_reticle_side_pos = w.def_f32(d::fHipReticleSidePos);
    f.ads_aim_pitch = w.def_f32(d::fAdsAimPitch);
    f.ads_crosshair_in_frac = w.def_f32(d::fAdsCrosshairInFrac);
    f.ads_crosshair_out_frac = w.def_f32(d::fAdsCrosshairOutFrac);
    f.ads_spread = w.def_f32(d::fAdsSpread);
    f.ads_zoom_in_frac = w.def_f32(d::fAdsZoomInFrac);
    f.ads_zoom_out_frac = w.def_f32(d::fAdsZoomOutFrac);
    f.aim_down_sight = w.def_u8(d::aimDownSight) != 0;
    f.hold_breath_to_steady = w.def_u8(d::canHoldBreath) != 0;
    f.no_ads_when_mag_empty = w.def_u8(d::noAdsWhenMagEmpty) != 0;
    f.rechamber_while_ads = w.def_u8(d::bRechamberWhileAds) != 0;
    f.position_reload_trans_ms = w.def_i32(d::iPositionReloadTransTime);
    f.ads_fire_only = w.def_u8(d::adsFireOnly) != 0;
    f.melee_damage = w.def_i32(d::iMeleeDamage);
    f.overlay_reticle = w.def_i32(d::overlay__reticle);
    f.overlay_interface = w.def_i32(d::overlayInterface);
    f.ads_overlay_width = w.def_f32(d::overlay__width);
    f.ads_overlay_height = w.def_f32(d::overlay__height);
    f.offhand_hold_is_cancelable = Some(w.def_u8(d::offhandHoldIsCancelable) != 0);
    f.select_requires_ammo = Some(w.def_u8(d::disableSwitchToWhenEmpty) != 0);
    f.move_speed_scale = w.def_f32(d::moveSpeedScale);
    f.ads_move_speed_scale = w.def_f32(d::adsMoveSpeedScale);
    f.sprint_duration_scale = w.def_f32(d::sprintDurationScale);
    f.ducked_ofs = w.def_vec3(d::vDuckedOfs);
    f.prone_ofs = w.def_vec3(d::vProneOfs);
    f.ads_bob_factor = w.def_f32(d::fAdsBobFactor);
    f.ads_view_bob_mult = w.def_f32(d::fAdsViewBobMult);
    f.movement = WeaponMovementOfsInputs {
        stand_move: w.def_vec3(d::vStandMove),
        stand_rot: w.def_vec3(d::vStandRot),
        strafe_move: w.def_vec3(d::strafeMove),
        strafe_rot: w.def_vec3(d::strafeRot),
        ducked_move: w.def_vec3(d::vDuckedMove),
        ducked_rot: w.def_vec3(d::vDuckedRot),
        prone_move: w.def_vec3(d::vProneMove),
        prone_rot: w.def_vec3(d::vProneRot),
        pos_move_rate: w.def_f32(d::fPosMoveRate),
        pos_prone_move_rate: w.def_f32(d::fPosProneMoveRate),
        stand_move_min_speed: w.def_f32(d::fStandMoveMinSpeed),
        ducked_move_min_speed: w.def_f32(d::fDuckedMoveMinSpeed),
        prone_move_min_speed: w.def_f32(d::fProneMoveMinSpeed),
        pos_rot_rate: w.def_f32(d::fPosRotRate),
        pos_prone_rot_rate: w.def_f32(d::fPosProneRotRate),
        ..WeaponMovementOfsInputs::default()
    };
    f.idle = WeaponIdleInputs {
        ads_idle_amount: w.def_f32(d::fAdsIdleAmount),
        hip_idle_amount: w.def_f32(d::fHipIdleAmount),
        ads_idle_speed: w.def_f32(d::adsIdleSpeed),
        hip_idle_speed: w.def_f32(d::hipIdleSpeed),
        idle_crouch_factor: w.def_f32(d::fIdleCrouchFactor),
        idle_prone_factor: w.def_f32(d::fIdleProneFactor),
    };
    f.penetrate_type = w.def_i32(d::penetrateType);
    f.rifle_bullet = w.def_u8(d::bRifleBullet) != 0;
    f.max_ammo = w.def_i32(d::iMaxAmmo);
    f.start_ammo = w.def_i32(d::iStartAmmo);
    f.damage = w.def_i32(d::damage);
    f.min_damage = w.def_i32(d::minDamage);
    f.min_player_damage = w.def_i32(d::minPlayerDamage);
    f.max_damage_range = w.def_f32(d::fMaxDamageRange);
    f.min_damage_range = w.def_f32(d::fMinDamageRange);
    f.kill_icon_ratio = w.def_i32(d::killIconRatio);
    f.flip_kill_icon = w.def_u8(d::flipKillIcon) != 0;
    f.reload_ammo_add = w.def_i32(d::iReloadAmmoAdd);
    f.reload_start_add = w.def_i32(d::iReloadStartAdd);
    f.no_partial_reload = w.def_u8(d::bNoPartialReload) != 0;
    f.bolt_action = w.def_u8(d::bBoltAction) != 0;
    f.segmented_reload = w.def_u8(d::bSegmentedReload) != 0;
    f.inherits_perks = w.def_u8(d::inheritsPerks) != 0;
    f.auto_aim_range = w.def_f32(d::autoAimRange);
    f.aim_assist_range = w.def_f32(d::aimAssistRange);
    f.aim_assist_range_ads = w.def_f32(d::aimAssistRangeAds);
    f.cook_off_hold = w.def_u8(d::bCookOffHold) != 0;
    f.clip_only = w.def_u8(d::bClipOnly) != 0;
    f.has_detonator = w.def_u8(d::hasDetonator) != 0;
    f.projectile_rotates = w.def_u8(d::rotate) != 0;
    f.timed_detonation = w.def_u8(d::timedDetonation) != 0;
    f.proj_impact_explode = w.def_u8(d::bProjImpactExplode) != 0;
    f.stick_to_players = w.def_u8(d::stickToPlayers) != 0;
    f.explosion_radius = w.def_i32(d::iExplosionRadius);
    f.explosion_radius_min = w.def_i32(d::iExplosionRadiusMin);
    f.explosion_inner_damage = w.def_i32(d::iExplosionInnerDamage);
    f.explosion_outer_damage = w.def_i32(d::iExplosionOuterDamage);
    f.damage_cone_angle = w.def_f32(d::damageConeAngle);
    f.stickiness = w.def_i32(d::stickiness);
    f.projectile_speed = w.def_i32(d::iProjectileSpeed);
    f.projectile_speed_up = w.def_i32(d::iProjectileSpeedUp);
    f.projectile_speed_forward = w.def_i32(d::iProjectileSpeedForward);
    f.projectile_activate_dist = w.def_i32(d::iProjectileActivateDist);
    f.require_lock_to_fire = w.def_u8(d::requireLockonToFire) != 0;
    f.location_damage_mult = w.location_damage.as_ref().map(|v| {
        let mut out = [1.0f32; 20];
        for (o, x) in out.iter_mut().zip(v.iter()) {
            *o = *x;
        }
        out
    });
    f.no_dual_wield = w.def_u8(d::noDualWield) != 0;
    f.kick.gun_max_pitch = w.def_f32(d::fGunMaxPitch);
    f.kick.gun_max_yaw = w.def_f32(d::fGunMaxYaw);
    f.kick.ads_gun_kick_reduced_kick_bullets = w.def_i32(d::adsGunKickReducedKickBullets);
    f.kick.ads_gun_kick_reduced_kick_percent = w.def_f32(d::adsGunKickReducedKickPercent);
    f.kick.ads_gun_kick_pitch_min = w.def_f32(d::fAdsGunKickPitchMin);
    f.kick.ads_gun_kick_pitch_max = w.def_f32(d::fAdsGunKickPitchMax);
    f.kick.ads_gun_kick_yaw_min = w.def_f32(d::fAdsGunKickYawMin);
    f.kick.ads_gun_kick_yaw_max = w.def_f32(d::fAdsGunKickYawMax);
    f.kick.ads_gun_kick_accel = w.def_f32(d::fAdsGunKickAccel);
    f.kick.ads_gun_kick_speed_max = w.def_f32(d::fAdsGunKickSpeedMax);
    f.kick.ads_gun_kick_speed_decay = w.def_f32(d::fAdsGunKickSpeedDecay);
    f.kick.ads_gun_kick_static_decay = w.def_f32(d::fAdsGunKickStaticDecay);
    f.kick.ads_view_kick_pitch_min = w.def_f32(d::fAdsViewKickPitchMin);
    f.kick.ads_view_kick_pitch_max = w.def_f32(d::fAdsViewKickPitchMax);
    f.kick.ads_view_kick_yaw_min = w.def_f32(d::fAdsViewKickYawMin);
    f.kick.ads_view_kick_yaw_max = w.def_f32(d::fAdsViewKickYawMax);
    f.kick.hip_gun_kick_reduced_kick_bullets = w.def_i32(d::hipGunKickReducedKickBullets);
    f.kick.hip_gun_kick_reduced_kick_percent = w.def_f32(d::hipGunKickReducedKickPercent);
    f.kick.hip_gun_kick_pitch_min = w.def_f32(d::fHipGunKickPitchMin);
    f.kick.hip_gun_kick_pitch_max = w.def_f32(d::fHipGunKickPitchMax);
    f.kick.hip_gun_kick_yaw_min = w.def_f32(d::fHipGunKickYawMin);
    f.kick.hip_gun_kick_yaw_max = w.def_f32(d::fHipGunKickYawMax);
    f.kick.hip_gun_kick_accel = w.def_f32(d::fHipGunKickAccel);
    f.kick.hip_gun_kick_speed_max = w.def_f32(d::fHipGunKickSpeedMax);
    f.kick.hip_gun_kick_speed_decay = w.def_f32(d::fHipGunKickSpeedDecay);
    f.kick.hip_gun_kick_static_decay = w.def_f32(d::fHipGunKickStaticDecay);
    f.kick.hip_view_kick_pitch_min = w.def_f32(d::fHipViewKickPitchMin);
    f.kick.hip_view_kick_pitch_max = w.def_f32(d::fHipViewKickPitchMax);
    f.kick.hip_view_kick_yaw_min = w.def_f32(d::fHipViewKickYawMin);
    f.kick.hip_view_kick_yaw_max = w.def_f32(d::fHipViewKickYawMax);
    f.sway = WeaponSwayFacts {
        sway_max_angle: w.def_f32(d::swayMaxAngle),
        sway_lerp_speed: w.def_f32(d::swayLerpSpeed),
        sway_pitch_scale: w.def_f32(d::swayPitchScale),
        sway_yaw_scale: w.def_f32(d::swayYawScale),
        sway_horiz_scale: w.def_f32(d::swayHorizScale),
        sway_vert_scale: w.def_f32(d::swayVertScale),
        sway_shell_shock_scale: w.def_f32(d::swayShellShockScale),
        ads_sway_max_angle: w.def_f32(d::adsSwayMaxAngle),
        ads_sway_lerp_speed: w.def_f32(d::adsSwayLerpSpeed),
        ads_sway_pitch_scale: w.def_f32(d::adsSwayPitchScale),
        ads_sway_yaw_scale: w.def_f32(d::adsSwayYawScale),
        ads_sway_horiz_scale: w.def_f32(d::adsSwayHorizScale),
        ads_sway_vert_scale: w.def_f32(d::adsSwayVertScale),
    };
    f
}

fn iw6_sounds(w: &Iw6WeaponView) -> WeaponSoundAliases {
    let s = |off: usize| w.sound(off);
    let bounce: Vec<&String> = w.sounds.iter().filter(|(o, _)| *o == d::bounceSound).map(|(_, n)| n).collect();
    WeaponSoundAliases {
        notetrack_convention: NotetrackConvention::SoundMap,
        fire: s(d::fireSound),
        fire_player: s(d::fireSoundPlayer),
        empty_fire: s(d::emptyFireSound),
        empty_fire_player: s(d::emptyFireSoundPlayer),
        melee_swipe: s(d::meleeSwipeSound),
        melee_swipe_player: s(d::meleeSwipeSoundPlayer),
        melee_hit: s(d::meleeHitSound),
        melee_miss: s(d::meleeMissSound),
        pickup: s(d::pickupSound),
        pickup_player: s(d::pickupSoundPlayer),
        ammo_pickup: s(d::ammoPickupSound),
        ammo_pickup_player: s(d::ammoPickupSoundPlayer),
        pullback: s(d::pullbackSound),
        pullback_player: s(d::pullbackSoundPlayer),
        reload: s(d::reloadSound),
        reload_player: s(d::reloadSoundPlayer),
        reload_empty: s(d::reloadEmptySound),
        reload_empty_player: s(d::reloadEmptySoundPlayer),
        reload_start: s(d::reloadStartSound),
        reload_start_player: s(d::reloadStartSoundPlayer),
        reload_end: s(d::reloadEndSound),
        reload_end_player: s(d::reloadEndSoundPlayer),
        rechamber: s(d::rechamberSound),
        rechamber_player: s(d::rechamberSoundPlayer),
        alt_switch: s(d::altSwitchSound),
        alt_switch_player: s(d::altSwitchSoundPlayer),
        raise: s(d::raiseSound),
        raise_player: s(d::raiseSoundPlayer),
        first_raise: s(d::firstRaiseSound),
        first_raise_player: s(d::firstRaiseSoundPlayer),
        putaway: s(d::putawaySound),
        putaway_player: s(d::putawaySoundPlayer),
        proj_explosion: s(d::projExplosionSound),
        projectile: s(d::projectileSound),
        proj_ignition_sound: s(d::projIgnitionSound),
        fire_loop: s(d::fireLoopSound),
        fire_loop_player: s(d::fireLoopSoundPlayer),
        fire_stop: s(d::fireStopSound),
        fire_stop_player: s(d::fireStopSoundPlayer),
        fire_last: s(d::fireLastSound),
        fire_last_player: s(d::fireLastSoundPlayer),
        notetrack_sound_map: w.notetrack_sounds.clone(),
        bounce: std::array::from_fn(|surf| bounce.get(surf).map(|n| (*n).clone()).filter(|n| !n.is_empty())),
        ..WeaponSoundAliases::default()
    }
}

impl WeaponCatalog {
    /// bo2mp: one Ghosts weapon (its complete definition with its
    /// `WeaponDef`), named without its `iw6_` prefix and `_mp` suffix
    /// (`iw6_ak12_mp` is `ak12`; a Black Ops II match gives it as
    /// `gh_ak12_mp`). False for a weapon without a name.
    pub fn capture_iw6(&mut self, w: &Iw6WeaponView) -> bool {
        let name = w.catalog_name();
        if name.is_empty() {
            return false;
        }
        let opt = |s: &str| (!s.is_empty()).then(|| s.to_owned());
        let combat_fx = WeaponCombatFx {
            view_flash_hint: w.effect(d::viewFlashEffect),
            world_flash_hint: w.effect(d::worldFlashEffect),
            view_shell_eject_hint: w.effect(d::viewShellEjectEffect),
            world_shell_eject_hint: w.effect(d::worldShellEjectEffect),
            view_last_shot_eject_hint: w.effect(d::viewLastShotEjectEffect),
            world_last_shot_eject_hint: w.effect(d::worldLastShotEjectEffect),
            explosion_hint: w.effect(d::projExplosionEffect),
            tracer_hint: w.tracer.clone(),
            ..WeaponCombatFx::default()
        };
        let reticle = WeaponReticleAssets {
            center_material: w.material(d::reticleCenter),
            side_material: w.material(d::reticleSide),
            center_authored: w.material(d::reticleCenter).is_some(),
            side_authored: w.material(d::reticleSide).is_some(),
            center_size: w.def_i32(d::iReticleCenterSize),
            side_size: w.def_i32(d::iReticleSideSize),
            ..WeaponReticleAssets::default()
        };
        // (The alternate mode's weapon: an underbarrel's own weapon.)
        let alternate = opt(&w.alt_weapon_name).map(|a| {
            let a = a.to_ascii_lowercase();
            let a = a.strip_suffix("_mp").unwrap_or(&a).to_owned();
            a.strip_prefix("iw6_").map_or(a.clone(), str::to_owned)
        });
        self.entries.push(CatalogWeapon {
            namespace: self.capture_ns,
            name,
            alternate_weapon: alternate,
            weap_def: None,
            display_name_key: opt(&w.display_name),
            reticle,
            hud_material_edges: WeaponHudMaterialEdges::default(),
            overlay_material: w.material(d::overlay__shader).or_else(|| w.material(d::overlay__shaderLowRes)),
            overlay_image: None,
            reticle_center_slot: None,
            reticle_side_slot: None,
            overlay_material_slot: None,
            scope_name: None,
            scope_rows: Default::default(),
            iw5_attachment_slots: std::array::from_fn(|_| None),
            iw5_reload_overrides: Vec::new(),
            iw5_anim_overrides: Vec::new(),
            iw5_fx_overrides: Vec::new(),
            iw5_notetrack_overrides: Vec::new(),
            hud_icon: w.material(d::hudIcon),
            hud_icon_slot: None,
            pickup_icon: w.material(d::pickupIcon),
            pickup_icon_slot: None,
            pickup_icon_image: None,
            pickup_icon_ratio: w.def_i32(d::pickupIconRatio),
            hud_icon_ratio: w.def_i32(d::hudIconRatio),
            hud_icon_image: None,
            dpad_icon: w.material(usize::MAX - c::dpadIcon),
            dpad_icon_image: None,
            dpad_icon_atlas: None,
            dpad_icon_ratio: w.complete_i32(c::dpadIconRatio),
            kill_icon: w.material(usize::MAX - c::killIcon),
            kill_icon_slot: None,
            kill_icon_image: None,
            proj_trail: w.effect(d::projTrailEffect),
            proj_trail_slot: None,
            proj_beacon: w.effect(d::projBeaconEffect),
            proj_beacon_slot: None,
            proj_ignition: w.effect(d::projIgnitionEffect),
            proj_ignition_slot: None,
            projectile_fx: WeaponProjectileFx::default(),
            gun_xmodel: w.gun_model.clone(),
            // BO2 draws the player character's arms with every weapon.
            hand_xmodel: None,
            world_model: w.world_model.clone(),
            projectile_model: w.model(d::projectileModel),
            rocket_model: w.model(d::rocketModel),
            knife_xmodel: w.model(d::knifeModel),
            sz_xanims: remap_iw6_sz_xanims(&w.xanims),
            sz_xanims_right: [const { None }; WEAPON_ANIM_SLOTS],
            sz_xanims_left: [const { None }; WEAPON_ANIM_SLOTS],
            dual_wield_weapon: None,
            hide_tags: w.hide_tags.clone(),
            ads_view_attachments: Vec::new(),
            view_attachments: Vec::new(),
            sounds: iw6_sounds(w),
            combat_fx,
            combat_slots: CombatFxSlots::default(),
            facts: iw6_body_facts(w),
        });
        true
    }
}
