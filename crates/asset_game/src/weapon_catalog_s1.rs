//! bo2mp: Call of Duty: Advanced Warfare weapons into the weapon catalog,
//! as guest guns in a Black Ops II match (`aw_<gun>_mp`).
//!
//! `S1WeaponView` carries a weapon's `WeaponDef` (AW keeps everything in
//! it: there is no complete definition) as bytes plus every name it points
//! at (read by `fastfile_s1`'s walk); this reads the fields the engine's
//! weapon rules, viewmodel and sounds use, by AW's offsets (the Ghosts
//! capture's fields under x64-zt's S1 names). AW's enums are Ghosts'
//! (type has a leading `none`; class and fire type IW4's numbers), and its
//! 180 animation slots are mapped to the catalog's.

use fastfile_s1::layout::WeaponDef as d;

use super::*;

/// One Advanced Warfare weapon as the walk read it.
#[derive(Clone, Debug, Default)]
pub struct S1WeaponView {
    /// `iw5_bal27_mp`.
    pub name: String,
    pub display_name: String,
    /// `WeaponDef` bytes.
    pub def: Vec<u8>,
    pub gun_model: Option<String>,
    pub world_model: Option<String>,
    /// The 180 animation slots (`weapAnimFiles_t`), empty for none.
    pub xanims: Vec<String>,
    pub hide_tags: Vec<String>,
    pub alt_weapon_name: String,
    /// (`WeaponDef` field offset, alias): every sound field.
    pub sounds: Vec<(usize, String)>,
    /// (`WeaponDef` field offset, effect).
    pub fx: Vec<(usize, String)>,
    /// (`WeaponDef` field offset, model).
    pub models: Vec<(usize, String)>,
    /// (`WeaponDef` field offset, material).
    pub materials: Vec<(usize, String)>,
    /// The notetrack sound map (notetrack, alias).
    pub notetrack_sounds: Vec<(String, String)>,
    /// `locationDamageMultipliers`: one per hit location.
    pub location_damage: Option<Vec<f32>>,
    pub tracer: Option<String>,
    /// bo2mp: the gun's own impact effects (`WeaponCombatFx::impact_row`),
    /// for an impact Black Ops II has nothing like; empty for none.
    pub impact_row: Vec<String>,
}

impl S1WeaponView {
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
        [
            self.def_f32(off),
            self.def_f32(off + 4),
            self.def_f32(off + 8),
        ]
    }
    fn sound(&self, off: usize) -> Option<String> {
        self.sounds
            .iter()
            .find(|(o, _)| *o == off)
            .map(|(_, n)| n.clone())
            .filter(|n| !n.is_empty())
    }
    fn effect(&self, off: usize) -> Option<String> {
        self.fx
            .iter()
            .find(|(o, _)| *o == off)
            .map(|(_, n)| n.clone())
            .filter(|n| !n.is_empty())
    }
    fn model(&self, off: usize) -> Option<String> {
        self.models
            .iter()
            .find(|(o, _)| *o == off)
            .map(|(_, n)| n.clone())
            .filter(|n| !n.is_empty())
    }
    fn material(&self, off: usize) -> Option<String> {
        self.materials
            .iter()
            .find(|(o, _)| *o == off)
            .map(|(_, n)| n.clone())
            .filter(|n| !n.is_empty())
    }

    /// The catalog's name for it: `iw5_bal27_mp` -> `bal27` (the guest name
    /// is then `aw_bal27_mp`).
    pub fn catalog_name(&self) -> String {
        let n = self.name.to_ascii_lowercase();
        let n = n.strip_suffix("_mp").unwrap_or(&n);
        n.strip_prefix("iw5_").unwrap_or(n).to_owned()
    }
}

/// AW `weapAnimFiles_t` slot -> the catalog's slot.
const S1_ANIM_SLOTS: [(usize, usize); 36] = [
    (1, weap_anim::IDLE),
    (2, weap_anim::EMPTY_IDLE),
    (3, weap_anim::FIRE),
    (4, weap_anim::HOLD_FIRE),
    (5, weap_anim::LASTSHOT),
    (6, weap_anim::RECHAMBER),
    // (AW splits the melee into swipe, hit and miss; the swipe is the
    // swing every melee plays.)
    (9, weap_anim::MELEE),
    (24, weap_anim::RELOAD),
    (25, weap_anim::RELOAD_EMPTY),
    (26, weap_anim::RELOAD_START),
    (27, weap_anim::RELOAD_END),
    (28, weap_anim_extra::RELOAD_QUICK),
    (29, weap_anim_extra::RELOAD_QUICK_EMPTY),
    (36, weap_anim::RAISE),
    (37, weap_anim::FIRST_RAISE),
    (38, weap_anim::BREACH_RAISE),
    (39, weap_anim::DROP),
    (40, weap_anim::ALT_RAISE),
    (41, weap_anim::ALT_DROP),
    (43, weap_anim::QUICK_RAISE),
    (44, weap_anim::QUICK_DROP),
    (45, weap_anim::EMPTY_RAISE),
    (46, weap_anim::EMPTY_DROP),
    (49, weap_anim::SPRINT_IN),
    (52, weap_anim::SPRINT_LOOP),
    (53, weap_anim::SPRINT_OUT),
    (57, weap_anim::STUNNED_START),
    (58, weap_anim::STUNNED_LOOP),
    (59, weap_anim::STUNNED_END),
    (61, weap_anim::DETONATE),
    (62, weap_anim::NIGHTVISION_WEAR),
    (63, weap_anim::NIGHTVISION_REMOVE),
    (64, weap_anim::ADS_FIRE),
    (65, weap_anim::ADS_LASTSHOT),
    (66, weap_anim::ADS_RECHAMBER),
    (128, weap_anim::ADS_UP),
];

fn remap_s1_sz_xanims(s1: &[String]) -> [Option<String>; WEAPON_ANIM_SLOTS] {
    let mut out = [const { None }; WEAPON_ANIM_SLOTS];
    for (src, dst) in S1_ANIM_SLOTS {
        if let Some(name) = s1.get(src).filter(|n| !n.is_empty()) {
            out[dst] = Some(name.to_ascii_lowercase());
        }
    }
    if let Some(name) = s1.get(129).filter(|n| !n.is_empty()) {
        out[weap_anim::ADS_DOWN] = Some(name.to_ascii_lowercase());
    }
    out
}

/// AW `weapType_t` starts with a `none` (as Ghosts'): bullet 1, grenade 2,
/// projectile 3, riot shield 4 are IW4's 0..3; energy weapons (5) fire as
/// bullets.
fn remap_s1_weap_type(raw: i32) -> i32 {
    match raw {
        1..=4 => raw - 1,
        _ => 0,
    }
}

/// AW `weapFireType_t`: full auto, single, bursts 2..4 as IW4; burst 5,
/// the double barrel and beyond fire single.
fn remap_s1_fire_type(raw: i32) -> i32 {
    match raw {
        0..=4 => raw,
        _ => 1,
    }
}

/// AW `ImpactType` -> Black Ops II's (the numbering its impact table and
/// the match's hit events read: none, bullet small, large, armour
/// piercing, extreme, shotgun, grenade bounce, grenade explode, rifle
/// grenade, rocket explode, ...). AW's own numbering (its impact table's
/// rows, measured on common_mp: row effects named `small_*`, `large_*`,
/// `20mm_*`, `expround_*`, `shotgun_*`, `energy_shotgun_*`, footsteps,
/// `frag_grenade_*`, `rocket_explosion_*`, `laser_small_*`) is IW's with
/// the energy shotgun (the Tac-19) and the laser (the EM1) added. Taken as
/// it was, the Tac-19's 7 read as Black Ops II's rifle grenade: every
/// pellet hit played a grenade explosion; the EM1's 12 a mortar shell.
pub fn remap_s1_impact_type(raw: i32) -> i32 {
    match raw {
        // Small, large, armour piercing.
        1..=3 => raw,
        // Explosive rounds (MORS, EPM3): Black Ops II's extreme bullet.
        4 => 4,
        // Shotgun, explosive shotgun, energy shotgun (Tac-19).
        5..=7 => 5,
        // Grenade bounce, grenade explode (frag, crossbow bolt, micro
        // drone), rocket (MAAWS, MAHEM), dud.
        8 => 6,
        9 => 7,
        10 => 9,
        11 => 11,
        // Laser (EM1), grapple and the rest: bullets.
        12 | 13 => 1,
        14.. => 2,
        _ => 0,
    }
}

fn s1_body_facts(w: &S1WeaponView) -> WeaponBodyFacts {
    let ads_in = w.def_i32(d::adsTransInTime);
    let ads_out = w.def_i32(d::adsTransOutTime);
    let mut f = WeaponBodyFacts {
        body_resolved: !w.def.is_empty(),
        clip_size: w.def_i32(d::clipSize),
        fire_time_ms: w.def_i32(d::stateTimers__fireTime),
        impact_type: remap_s1_impact_type(w.def_i32(d::impactType)),
        alternate_raise_time_ms: w.def_i32(d::stateTimers__altRaiseTime),
        ads_zoom_fov: w.def_f32(d::adsZoomFov),
        ads_in_rate: if ads_in > 0 { 1.0 / ads_in as f32 } else { 0.0 },
        ads_out_rate: if ads_out > 0 {
            1.0 / ads_out as f32
        } else {
            0.0
        },
        // (AW has no penetration multiplier: its depth table scales as is.)
        penetrate_multiplier: 1.0,
        motion_tracker: w.def_u8(d::motionTracker) != 0,
        select_requires_ammo: Some(false),
        ..WeaponBodyFacts::default()
    };
    let dof = [w.def_f32(d::adsDofStart), w.def_f32(d::adsDofEnd)];
    f.ads_dof = (dof[1] > 0.0).then_some(dof);
    f.kick.f_ads_view_kick_center_speed = w.def_f32(d::adsViewKickCenterSpeed);
    f.kick.f_hip_view_kick_center_speed = w.def_f32(d::hipViewKickCenterSpeed);
    if w.def.is_empty() {
        return f;
    }
    let t = |off: usize| w.def_i32(off);
    f.raise_time_ms = t(d::stateTimers__raiseTime);
    f.drop_time_ms = t(d::stateTimers__dropTime);
    f.alternate_drop_time_ms = t(d::stateTimers__altDropTime);
    f.fire_delay_ms = t(d::stateTimers__fireDelay);
    f.hold_fire_time_ms = t(d::stateTimers__holdFireTime);
    f.melee_time_ms = t(d::stateTimers__meleeTime);
    f.melee_delay_ms = t(d::stateTimers__meleeDelay);
    f.melee_charge_time_ms = t(d::stateTimers__meleeChargeTime);
    f.melee_charge_delay_ms = t(d::stateTimers__meleeChargeDelay);
    f.rechamber_time_ms = t(d::stateTimers__rechamberTime);
    f.rechamber_bolt_time_ms = t(d::stateTimers__rechamberBoltTime);
    f.reload_time_ms = t(d::stateTimers__reloadTime);
    f.reload_empty_time_ms = t(d::stateTimers__reloadEmptyTime);
    f.reload_show_rocket_time_ms = t(d::stateTimers__reloadShowRocketTime);
    f.reload_add_time_ms = t(d::stateTimers__reloadAddTime);
    f.reload_empty_add_time_ms = t(d::stateTimers__reloadEmptyAddTime);
    f.reload_start_time_ms = t(d::stateTimers__reloadStartTime);
    f.reload_start_add_time_ms = t(d::stateTimers__reloadStartAddTime);
    f.reload_end_time_ms = t(d::stateTimers__reloadEndTime);
    f.quick_raise_time_ms = t(d::stateTimers__quickRaiseTime);
    f.quick_drop_time_ms = t(d::stateTimers__quickDropTime);
    f.sprint_raise_time_ms = t(d::stateTimers__sprintInTime);
    f.sprint_loop_time_ms = t(d::stateTimers__sprintLoopTime);
    f.sprint_drop_time_ms = t(d::stateTimers__sprintOutTime);
    f.night_vision_wear_time = t(d::stateTimers__nightVisionWearTime);
    f.fuse_time_ms = t(d::stateTimers__fuseTime);
    f.detonate_delay_ms = t(d::stateTimers__detonateDelay);
    f.detonate_time_ms = t(d::stateTimers__detonateTime);
    f.weap_type = remap_s1_weap_type(w.def_i32(d::weapType));
    f.player_anim_type = i32::from(w.def_u8(d::playerAnimType));
    f.weap_class = match w.def_i32(d::weapClass) {
        k @ 0..=11 => k,
        _ => 0,
    };
    f.offhand_class = w.def_i32(d::offhandClass);
    f.inventory_type = match w.def_i32(d::inventoryType) {
        k @ 0..=3 => k,
        _ => 0,
    };
    f.fire_type = remap_s1_fire_type(w.def_i32(d::fireType));
    f.shots_per_fire = w.def_i32(d::shotCount);
    f.ammo_counter_clip = w.def_i32(d::ammoCounterClip);
    f.low_ammo_warning_threshold = w.def_f32(d::lowAmmoWarningThreshold);
    apply_leftover_hip_spread(
        &mut f,
        leftover_hip_spread_block(|off| w.def_f32(off), d::hipSpreadStandMin),
    );
    f.i_reticle_side_size = w.def_i32(d::reticleSideSize);
    f.i_reticle_min_ofs = w.def_i32(d::reticleMinOfs);
    f.hip_reticle_side_pos = w.def_f32(d::hipReticleSidePos);
    f.ads_aim_pitch = w.def_f32(d::adsAimPitch);
    f.ads_crosshair_in_frac = w.def_f32(d::adsCrosshairInFrac);
    f.ads_crosshair_out_frac = w.def_f32(d::adsCrosshairOutFrac);
    f.ads_spread = w.def_f32(d::adsSpread);
    f.ads_zoom_in_frac = w.def_f32(d::adsZoomInFrac);
    f.ads_zoom_out_frac = w.def_f32(d::adsZoomOutFrac);
    f.aim_down_sight = w.def_u8(d::aimDownSight) != 0;
    f.hold_breath_to_steady = w.def_u8(d::canHoldBreath) != 0;
    f.no_ads_when_mag_empty = w.def_u8(d::noAdsWhenMagEmpty) != 0;
    f.rechamber_while_ads = w.def_u8(d::rechamberWhileAds) != 0;
    f.position_reload_trans_ms = w.def_i32(d::positionReloadTransTime);
    f.melee_damage = w.def_i32(d::meleeDamage);
    f.overlay_reticle = w.def_i32(d::overlay__reticle);
    f.overlay_interface = w.def_i32(d::overlayInterface);
    f.ads_overlay_width = w.def_f32(d::overlay__width);
    f.ads_overlay_height = w.def_f32(d::overlay__height);
    f.offhand_hold_is_cancelable = Some(w.def_u8(d::offhandHoldIsCancelable) != 0);
    f.select_requires_ammo = Some(w.def_u8(d::disableSwitchToWhenEmpty) != 0);
    f.move_speed_scale = w.def_f32(d::moveSpeedScale);
    f.ads_move_speed_scale = w.def_f32(d::adsMoveSpeedScale);
    f.sprint_duration_scale = w.def_f32(d::sprintDurationScale);
    f.ducked_ofs = w.def_vec3(d::duckedOfs);
    f.prone_ofs = w.def_vec3(d::proneOfs);
    f.ads_bob_factor = w.def_f32(d::adsBobFactor);
    f.ads_view_bob_mult = w.def_f32(d::adsViewBobMult);
    f.movement = WeaponMovementOfsInputs {
        stand_move: w.def_vec3(d::standMove),
        stand_rot: w.def_vec3(d::standRot),
        strafe_move: w.def_vec3(d::strafeMove),
        strafe_rot: w.def_vec3(d::strafeRot),
        ducked_move: w.def_vec3(d::duckedMove),
        ducked_rot: w.def_vec3(d::duckedRot),
        prone_move: w.def_vec3(d::proneMove),
        prone_rot: w.def_vec3(d::proneRot),
        pos_move_rate: w.def_f32(d::posMoveRate),
        pos_prone_move_rate: w.def_f32(d::posProneMoveRate),
        stand_move_min_speed: w.def_f32(d::standMoveMinSpeed),
        ducked_move_min_speed: w.def_f32(d::duckedMoveMinSpeed),
        prone_move_min_speed: w.def_f32(d::proneMoveMinSpeed),
        pos_rot_rate: w.def_f32(d::posRotRate),
        pos_prone_rot_rate: w.def_f32(d::posProneRotRate),
        ..WeaponMovementOfsInputs::default()
    };
    f.idle = WeaponIdleInputs {
        ads_idle_amount: w.def_f32(d::adsIdleAmount),
        hip_idle_amount: w.def_f32(d::hipIdleAmount),
        ads_idle_speed: w.def_f32(d::adsIdleSpeed),
        hip_idle_speed: w.def_f32(d::hipIdleSpeed),
        idle_crouch_factor: w.def_f32(d::idleCrouchFactor),
        idle_prone_factor: w.def_f32(d::idleProneFactor),
    };
    f.penetrate_type = w.def_i32(d::penetrateType);
    f.rifle_bullet = w.def_u8(d::rifleBullet) != 0;
    f.max_ammo = w.def_i32(d::maxAmmo);
    f.start_ammo = w.def_i32(d::startAmmo);
    f.damage = w.def_i32(d::damage);
    f.min_damage = w.def_i32(d::minDamage);
    f.min_player_damage = w.def_i32(d::minPlayerDamage);
    f.max_damage_range = w.def_f32(d::maxDamageRange);
    f.min_damage_range = w.def_f32(d::minDamageRange);
    f.kill_icon_ratio = w.def_i32(d::killIconRatio);
    f.flip_kill_icon = w.def_u8(d::flipKillIcon) != 0;
    f.reload_ammo_add = w.def_i32(d::reloadAmmoAdd);
    f.reload_start_add = w.def_i32(d::reloadStartAdd);
    f.no_partial_reload = w.def_u8(d::noPartialReload) != 0;
    f.bolt_action = w.def_u8(d::boltAction) != 0;
    f.segmented_reload = w.def_u8(d::segmentedReload) != 0;
    f.inherits_perks = w.def_u8(d::inheritsPerks) != 0;
    f.auto_aim_range = w.def_f32(d::autoAimRange);
    f.aim_assist_range = w.def_f32(d::aimAssistRange);
    f.aim_assist_range_ads = w.def_f32(d::aimAssistRangeAds);
    f.cook_off_hold = w.def_u8(d::cookOffHold) != 0;
    f.clip_only = w.def_u8(d::clipOnly) != 0;
    f.projectile_rotates = w.def_u8(d::rotate) != 0;
    f.timed_detonation = w.def_u8(d::timedDetonation) != 0;
    f.proj_impact_explode = w.def_u8(d::projImpactExplode) != 0;
    f.stick_to_players = w.def_u8(d::stickToPlayers) != 0;
    f.explosion_radius = w.def_i32(d::explosionRadius);
    f.explosion_radius_min = w.def_i32(d::explosionRadiusMin);
    f.explosion_inner_damage = w.def_i32(d::explosionInnerDamage);
    f.explosion_outer_damage = w.def_i32(d::explosionOuterDamage);
    f.damage_cone_angle = w.def_f32(d::damageConeAngle);
    f.stickiness = w.def_i32(d::stickiness);
    f.projectile_speed = w.def_i32(d::projectileSpeed);
    f.projectile_speed_up = w.def_i32(d::projectileSpeedUp);
    f.projectile_speed_forward = w.def_i32(d::projectileSpeedForward);
    f.projectile_activate_dist = w.def_i32(d::projectileActivateDist);
    f.require_lock_to_fire = w.def_u8(d::requireLockonToFire) != 0;
    // (AW's IW order.)
    f.location_damage_mult = w.location_damage.as_ref().map(|v| weapon_iw4::hitloc_iw_to_t6(v));
    f.no_dual_wield = w.def_u8(d::noDualWield) != 0;
    f.kick.gun_max_pitch = w.def_f32(d::gunMaxPitch);
    f.kick.gun_max_yaw = w.def_f32(d::gunMaxYaw);
    f.kick.ads_gun_kick_reduced_kick_bullets = w.def_i32(d::adsGunKickReducedKickBullets);
    f.kick.ads_gun_kick_reduced_kick_percent = w.def_f32(d::adsGunKickReducedKickPercent);
    f.kick.ads_gun_kick_pitch_min = w.def_f32(d::adsGunKickPitchMin);
    f.kick.ads_gun_kick_pitch_max = w.def_f32(d::adsGunKickPitchMax);
    f.kick.ads_gun_kick_yaw_min = w.def_f32(d::adsGunKickYawMin);
    f.kick.ads_gun_kick_yaw_max = w.def_f32(d::adsGunKickYawMax);
    f.kick.ads_gun_kick_accel = w.def_f32(d::adsGunKickAccel);
    f.kick.ads_gun_kick_speed_max = w.def_f32(d::adsGunKickSpeedMax);
    f.kick.ads_gun_kick_speed_decay = w.def_f32(d::adsGunKickSpeedDecay);
    f.kick.ads_gun_kick_static_decay = w.def_f32(d::adsGunKickStaticDecay);
    f.kick.ads_view_kick_pitch_min = w.def_f32(d::adsViewKickPitchMin);
    f.kick.ads_view_kick_pitch_max = w.def_f32(d::adsViewKickPitchMax);
    f.kick.ads_view_kick_yaw_min = w.def_f32(d::adsViewKickYawMin);
    f.kick.ads_view_kick_yaw_max = w.def_f32(d::adsViewKickYawMax);
    f.kick.hip_gun_kick_reduced_kick_bullets = w.def_i32(d::hipGunKickReducedKickBullets);
    f.kick.hip_gun_kick_reduced_kick_percent = w.def_f32(d::hipGunKickReducedKickPercent);
    f.kick.hip_gun_kick_pitch_min = w.def_f32(d::hipGunKickPitchMin);
    f.kick.hip_gun_kick_pitch_max = w.def_f32(d::hipGunKickPitchMax);
    f.kick.hip_gun_kick_yaw_min = w.def_f32(d::hipGunKickYawMin);
    f.kick.hip_gun_kick_yaw_max = w.def_f32(d::hipGunKickYawMax);
    f.kick.hip_gun_kick_accel = w.def_f32(d::hipGunKickAccel);
    f.kick.hip_gun_kick_speed_max = w.def_f32(d::hipGunKickSpeedMax);
    f.kick.hip_gun_kick_speed_decay = w.def_f32(d::hipGunKickSpeedDecay);
    f.kick.hip_gun_kick_static_decay = w.def_f32(d::hipGunKickStaticDecay);
    f.kick.hip_view_kick_pitch_min = w.def_f32(d::hipViewKickPitchMin);
    f.kick.hip_view_kick_pitch_max = w.def_f32(d::hipViewKickPitchMax);
    f.kick.hip_view_kick_yaw_min = w.def_f32(d::hipViewKickYawMin);
    f.kick.hip_view_kick_yaw_max = w.def_f32(d::hipViewKickYawMax);
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

fn s1_sounds(w: &S1WeaponView) -> WeaponSoundAliases {
    let s = |off: usize| w.sound(off);
    let bounce: Vec<&String> = w
        .sounds
        .iter()
        .filter(|(o, _)| *o == d::bounceSound)
        .map(|(_, n)| n)
        .collect();
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
        bounce: std::array::from_fn(|surf| {
            bounce
                .get(surf)
                .map(|n| (*n).clone())
                .filter(|n| !n.is_empty())
        }),
        ..WeaponSoundAliases::default()
    }
}

impl WeaponCatalog {
    /// bo2mp: one Advanced Warfare weapon, named without its `iw5_` prefix
    /// and `_mp` suffix (`iw5_bal27_mp` is `bal27`; a Black Ops II match
    /// gives it as `aw_bal27_mp`). False for a weapon without a name.
    pub fn capture_s1(&mut self, w: &S1WeaponView) -> bool {
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
            impact_row: w.impact_row.clone(),
            ..WeaponCombatFx::default()
        };
        let reticle = WeaponReticleAssets {
            center_material: w.material(d::reticleCenter),
            side_material: w.material(d::reticleSide),
            center_authored: w.material(d::reticleCenter).is_some(),
            side_authored: w.material(d::reticleSide).is_some(),
            center_size: w.def_i32(d::reticleCenterSize),
            side_size: w.def_i32(d::reticleSideSize),
            ..WeaponReticleAssets::default()
        };
        // (The alternate mode's weapon: an underbarrel's own weapon.)
        let alternate = opt(&w.alt_weapon_name).map(|a| {
            let a = a.to_ascii_lowercase();
            let a = a.strip_suffix("_mp").unwrap_or(&a).to_owned();
            a.strip_prefix("iw5_").map_or(a.clone(), str::to_owned)
        });
        self.entries.push(CatalogWeapon {
            world_attachments: Vec::new(),
            spawned_grenade: None,
            namespace: self.capture_ns,
            name,
            alternate_weapon: alternate,
            weap_def: None,
            display_name_key: opt(&w.display_name),
            reticle,
            hud_material_edges: WeaponHudMaterialEdges::default(),
            overlay_material: w
                .material(d::overlay__shader)
                .or_else(|| w.material(d::overlay__shaderLowRes)),
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
            dpad_icon: w.material(d::dpadIcon),
            dpad_icon_image: None,
            dpad_icon_atlas: None,
            dpad_icon_ratio: w.def_i32(d::dpadIconRatio),
            kill_icon: w.material(d::killIcon),
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
            sz_xanims: remap_s1_sz_xanims(&w.xanims),
            sz_xanims_right: [const { None }; WEAPON_ANIM_SLOTS],
            sz_xanims_left: [const { None }; WEAPON_ANIM_SLOTS],
            dual_wield_weapon: None,
            hide_tags: w.hide_tags.clone(),
            ads_view_attachments: Vec::new(),
            view_attachments: Vec::new(),
            sounds: s1_sounds(w),
            combat_fx,
            combat_slots: CombatFxSlots::default(),
            facts: s1_body_facts(w),
        });
        true
    }
}
