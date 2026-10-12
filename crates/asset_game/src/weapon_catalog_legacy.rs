//! bo2mp: Call of Duty 4 (IW3) and World at War (T4) weapons into the weapon
//! catalog, as guest guns in a Black Ops II match (`cod4_ak47_mp`,
//! `waw_mp40_mp+silencer`).
//!
//! Both games keep a weapon in one `WeaponDef` (Modern Warfare 2 split it
//! into a complete definition and a body later). The `iw3` and `t4` readers
//! load it through their schemas, so `LegacyWeaponView` carries its struct
//! image with every member's offset by name, and the names its pointers
//! lead to; the fields read here are named as both games (and IW4) name
//! them. World at War's `WeaponDef` is CoD4's plus overheating, mounting,
//! flame and deploy fields, so the same reads serve both. Their enums follow
//! CoD4's numbering, which differs from IW4's for the weapon class and the
//! impact type; their animation slots are CoD4's (World at War adds deploy
//! and breakdown after the sprint ones).

use std::collections::HashMap;

use super::*;

/// Which of the two games a view is from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegacyGame {
    /// Call of Duty 4.
    #[default]
    Iw3,
    /// World at War.
    T4,
}

/// One CoD4 or World at War weapon as its reader loaded it.
#[derive(Clone, Debug, Default)]
pub struct LegacyWeaponView {
    pub game: LegacyGame,
    /// `ak47_mp`.
    pub name: String,
    pub display_name: String,
    /// The `WeaponDef` struct image.
    pub def: Vec<u8>,
    /// Every `WeaponDef` member's offset, by name.
    pub offsets: HashMap<String, usize>,
    /// The first view and world models.
    pub gun_model: Option<String>,
    pub world_model: Option<String>,
    /// `szXAnims`, as authored (empty for none).
    pub xanims: Vec<String>,
    pub hide_tags: Vec<String>,
    pub alt_weapon_name: String,
    /// (member, alias) for every sound member that names one.
    pub sounds: HashMap<String, String>,
    /// `bounceSound`, one per surface type.
    pub bounce: Vec<String>,
    /// (member, name) for every effect, model and material member.
    pub assets: HashMap<String, String>,
    /// The notetrack sound map (notetrack, alias).
    pub notetrack_sounds: Vec<(String, String)>,
}

impl LegacyWeaponView {
    fn rd<const N: usize>(&self, member: &str, add: usize) -> Option<[u8; N]> {
        let at = *self.offsets.get(member)? + add;
        self.def.get(at..at + N)?.try_into().ok()
    }
    /// An `int` (or enum, or `int`-sized bool) member.
    pub fn i(&self, member: &str) -> i32 {
        self.rd::<4>(member, 0).map_or(0, i32::from_le_bytes)
    }
    pub fn f(&self, member: &str) -> f32 {
        self.rd::<4>(member, 0).map_or(0.0, f32::from_le_bytes)
    }
    /// Element `k` of a `float[]` member.
    pub fn fk(&self, member: &str, k: usize) -> f32 {
        self.rd::<4>(member, k * 4).map_or(0.0, f32::from_le_bytes)
    }
    pub fn v3(&self, member: &str) -> [f32; 3] {
        std::array::from_fn(|k| self.fk(member, k))
    }
    fn b(&self, member: &str) -> bool {
        self.i(member) != 0
    }
    fn sound(&self, member: &str) -> Option<String> {
        self.sounds.get(member).filter(|s| !s.is_empty()).cloned()
    }
    fn asset(&self, member: &str) -> Option<String> {
        self.assets
            .get(member)
            .map(|s| s.trim_start_matches(',').to_owned())
            .filter(|s| !s.is_empty())
    }

    /// The catalog's name for it: `ak47_mp` -> `ak47` (the guest name is
    /// then `cod4_ak47_mp`).
    pub fn catalog_name(&self) -> String {
        let n = self.name.to_ascii_lowercase();
        n.strip_suffix("_mp").unwrap_or(&n).to_owned()
    }
}

/// CoD4 `weapAnimFiles_t` slot -> the catalog's (IW4) slot. World at War's
/// slots are CoD4's until the sprint ones; then it has deploy and breakdown
/// (25, 26), so everything after moves up two.
fn legacy_anim_slots(game: LegacyGame) -> Vec<(usize, usize)> {
    let mut pairs = vec![
        (1, weap_anim::IDLE),
        (2, weap_anim::EMPTY_IDLE),
        (3, weap_anim::FIRE),
        (4, weap_anim::HOLD_FIRE),
        (5, weap_anim::LASTSHOT),
        (6, weap_anim::RECHAMBER),
        (7, weap_anim::MELEE),
        (8, weap_anim::MELEE_CHARGE),
        (9, weap_anim::RELOAD),
        (10, weap_anim::RELOAD_EMPTY),
        (11, weap_anim::RELOAD_START),
        (12, weap_anim::RELOAD_END),
        (13, weap_anim::RAISE),
        (14, weap_anim::FIRST_RAISE),
        (15, weap_anim::DROP),
        (16, weap_anim::ALT_RAISE),
        (17, weap_anim::ALT_DROP),
        (18, weap_anim::QUICK_RAISE),
        (19, weap_anim::QUICK_DROP),
        (20, weap_anim::EMPTY_RAISE),
        (21, weap_anim::EMPTY_DROP),
        (22, weap_anim::SPRINT_IN),
        (23, weap_anim::SPRINT_LOOP),
        (24, weap_anim::SPRINT_OUT),
    ];
    let after = [
        weap_anim::DETONATE,
        weap_anim::NIGHTVISION_WEAR,
        weap_anim::NIGHTVISION_REMOVE,
        weap_anim::ADS_FIRE,
        weap_anim::ADS_LASTSHOT,
        weap_anim::ADS_RECHAMBER,
        weap_anim::ADS_UP,
        weap_anim::ADS_DOWN,
    ];
    let first = match game {
        LegacyGame::Iw3 => 25,
        LegacyGame::T4 => 27,
    };
    pairs.extend(after.into_iter().enumerate().map(|(k, dst)| (first + k, dst)));
    pairs
}

fn remap_legacy_sz_xanims(game: LegacyGame, src: &[String]) -> [Option<String>; WEAPON_ANIM_SLOTS] {
    let mut out = [const { None }; WEAPON_ANIM_SLOTS];
    for (from, to) in legacy_anim_slots(game) {
        if let Some(name) = src.get(from).filter(|n| !n.is_empty()) {
            out[to] = Some(name.to_ascii_lowercase());
        }
    }
    out
}

/// CoD4's and World at War's `weapClass_t` (rifle, mg, smg, spread,
/// pistol, grenade, rocket launcher, turret, non-player, then WaW's gas and
/// both games' item) as IW4's (which adds sniper, throwing knife).
fn remap_legacy_weap_class(game: LegacyGame, raw: i32) -> i32 {
    match (game, raw) {
        (_, 0) => 0,
        (_, k @ 1..=7) => k + 1,
        (_, 8) => 10,
        (LegacyGame::Iw3, 9) => 11,
        // (World at War's gas is thrown as a grenade.)
        (LegacyGame::T4, 9) => 6,
        (LegacyGame::T4, 10) => 11,
        _ => 0,
    }
}

/// `weapType_t`: bullet, grenade, projectile as IW4; binoculars and World
/// at War's gas, bomb and mine are not fired like guns.
fn remap_legacy_weap_type(raw: i32) -> i32 {
    match raw {
        0..=2 => raw,
        // (Gas canisters, mines: thrown.)
        4 | 6 => 1,
        _ => 0,
    }
}

/// `ImpactType` as IW4's (which adds the explosive bullet and shotgun).
fn remap_legacy_impact_type(game: LegacyGame, raw: i32) -> i32 {
    match (game, raw) {
        (_, 0..=3) => raw,
        (_, 4) => 5,
        (_, 5) => 7,
        (_, 6) => 8,
        (LegacyGame::Iw3, 7) => 9,
        (LegacyGame::Iw3, 8) => 10,
        // World at War: rifle grenade, rocket, dud, mortar and tank shells.
        (LegacyGame::T4, 7) => 8,
        (LegacyGame::T4, 8) => 9,
        (LegacyGame::T4, 9) => 10,
        (LegacyGame::T4, _) => 9,
        _ => 0,
    }
}

fn legacy_body_facts(w: &LegacyWeaponView) -> WeaponBodyFacts {
    let ads_in = w.i("iAdsTransInTime");
    let ads_out = w.i("iAdsTransOutTime");
    let weap_class = remap_legacy_weap_class(w.game, w.i("weapClass"));
    let mut f = WeaponBodyFacts {
        body_resolved: !w.def.is_empty(),
        clip_size: w.i("iClipSize"),
        fire_time_ms: w.i("iFireTime"),
        impact_type: remap_legacy_impact_type(w.game, w.i("impactType")),
        alternate_raise_time_ms: w.i("iAltRaiseTime"),
        ads_zoom_fov: w.f("fAdsZoomFov"),
        ads_in_rate: if ads_in > 0 { 1.0 / ads_in as f32 } else { 0.0 },
        ads_out_rate: if ads_out > 0 { 1.0 / ads_out as f32 } else { 0.0 },
        select_requires_ammo: Some(false),
        ..WeaponBodyFacts::default()
    };
    let dof = [w.f("adsDofStart"), w.f("adsDofEnd")];
    f.ads_dof = (dof[1] > 0.0).then_some(dof);
    f.kick.f_ads_view_kick_center_speed = w.f("fAdsViewKickCenterSpeed");
    f.kick.f_hip_view_kick_center_speed = w.f("fHipViewKickCenterSpeed");
    f.raise_time_ms = w.i("iRaiseTime");
    f.drop_time_ms = w.i("iDropTime");
    f.alternate_drop_time_ms = w.i("iAltDropTime");
    f.fire_delay_ms = w.i("iFireDelay");
    f.hold_fire_time_ms = w.i("iHoldFireTime");
    f.melee_time_ms = w.i("iMeleeTime");
    f.melee_delay_ms = w.i("iMeleeDelay");
    f.melee_charge_time_ms = w.i("meleeChargeTime");
    f.melee_charge_delay_ms = w.i("meleeChargeDelay");
    f.rechamber_time_ms = w.i("iRechamberTime");
    f.rechamber_bolt_time_ms = w.i("iRechamberBoltTime");
    f.reload_time_ms = w.i("iReloadTime");
    f.reload_empty_time_ms = w.i("iReloadEmptyTime");
    f.reload_show_rocket_time_ms = w.i("reloadShowRocketTime");
    f.reload_add_time_ms = w.i("iReloadAddTime");
    // (CoD4 has no separate empty add time: the add time serves both.)
    f.reload_empty_add_time_ms = match w.game {
        LegacyGame::T4 => w.i("reloadEmptyAddTime"),
        LegacyGame::Iw3 => w.i("iReloadAddTime"),
    };
    f.reload_start_time_ms = w.i("iReloadStartTime");
    f.reload_start_add_time_ms = w.i("iReloadStartAddTime");
    f.reload_end_time_ms = w.i("iReloadEndTime");
    f.quick_raise_time_ms = w.i("quickRaiseTime");
    f.quick_drop_time_ms = w.i("quickDropTime");
    f.sprint_raise_time_ms = w.i("sprintInTime");
    f.sprint_loop_time_ms = w.i("sprintLoopTime");
    f.sprint_drop_time_ms = w.i("sprintOutTime");
    f.night_vision_wear_time = w.i("nightVisionWearTime");
    f.fuse_time_ms = w.i("fuseTime");
    f.detonate_delay_ms = w.i("iDetonateDelay");
    f.detonate_time_ms = w.i("iDetonateTime");
    f.weap_type = remap_legacy_weap_type(w.i("weapType"));
    // (CoD4's player animation types are IW4's first fifteen; World at
    // War's later ones, flamethrower and deployed, play as a rifle.)
    f.player_anim_type = match w.i("playerAnimType") {
        k @ 0..=14 => k,
        _ => 4,
    };
    // (CoD4 and WaW snipers are rifles with a scope overlay; IW4 has a
    // sniper class.)
    f.weap_class = if weap_class == 0 && w.i("overlayReticle") != 0 { 1 } else { weap_class };
    f.offhand_class = w.i("offhandClass");
    // (CoD4's RPG is an inventory item and World at War's bazooka an
    // offhand; as Black Ops II launchers they are held like guns.)
    f.inventory_type = match w.i("inventoryType") {
        _ if weap_class == 7 => 0,
        k @ 0..=3 => k,
        _ => 0,
    };
    f.fire_type = match w.i("fireType") {
        k @ 0..=4 => k,
        _ => 1,
    };
    f.shots_per_fire = w.i("shotCount");
    f.ammo_counter_clip = w.i("ammoCounterClip");
    f.low_ammo_warning_threshold = w.f("lowAmmoWarningThreshold");
    // (Twelve floats from `fHipSpreadStandMin`, laid out as IW4's.)
    let at = |k: usize| w.fk("fHipSpreadStandMin", k);
    apply_leftover_hip_spread(&mut f, std::array::from_fn(at));
    f.i_reticle_side_size = w.i("iReticleSideSize");
    f.i_reticle_min_ofs = w.i("iReticleMinOfs");
    f.hip_reticle_side_pos = w.f("fHipReticleSidePos");
    f.ads_aim_pitch = w.f("fAdsAimPitch");
    f.ads_crosshair_in_frac = w.f("fAdsCrosshairInFrac");
    f.ads_crosshair_out_frac = w.f("fAdsCrosshairOutFrac");
    f.ads_spread = w.f("fAdsSpread");
    f.ads_zoom_in_frac = w.f("fAdsZoomInFrac");
    f.ads_zoom_out_frac = w.f("fAdsZoomOutFrac");
    f.aim_down_sight = w.b("aimDownSight");
    // (CoD4's snipers hold breath: their scope overlays say so.)
    f.hold_breath_to_steady = w.i("overlayReticle") != 0;
    f.no_ads_when_mag_empty = w.b("noAdsWhenMagEmpty");
    f.rechamber_while_ads = w.b("bRechamberWhileAds");
    f.position_reload_trans_ms = w.i("iPositionReloadTransTime");
    f.ads_fire_only = w.b("adsFireOnly");
    f.melee_damage = w.i("iMeleeDamage");
    f.overlay_reticle = w.i("overlayReticle");
    f.overlay_interface = w.i("overlayInterface");
    f.ads_overlay_width = w.f("overlayWidth");
    f.ads_overlay_height = w.f("overlayHeight");
    f.move_speed_scale = w.f("moveSpeedScale");
    f.ads_move_speed_scale = w.f("adsMoveSpeedScale");
    f.sprint_duration_scale = w.f("sprintDurationScale");
    f.ducked_ofs = w.v3("vDuckedOfs");
    f.prone_ofs = w.v3("vProneOfs");
    f.ads_bob_factor = w.f("fAdsBobFactor");
    f.ads_view_bob_mult = w.f("fAdsViewBobMult");
    f.movement = WeaponMovementOfsInputs {
        stand_move: w.v3("vStandMove"),
        stand_rot: w.v3("vStandRot"),
        ducked_move: w.v3("vDuckedMove"),
        ducked_rot: w.v3("vDuckedRot"),
        prone_move: w.v3("vProneMove"),
        prone_rot: w.v3("vProneRot"),
        pos_move_rate: w.f("fPosMoveRate"),
        pos_prone_move_rate: w.f("fPosProneMoveRate"),
        stand_move_min_speed: w.f("fStandMoveMinSpeed"),
        ducked_move_min_speed: w.f("fDuckedMoveMinSpeed"),
        prone_move_min_speed: w.f("fProneMoveMinSpeed"),
        pos_rot_rate: w.f("fPosRotRate"),
        pos_prone_rot_rate: w.f("fPosProneRotRate"),
        ..WeaponMovementOfsInputs::default()
    };
    f.idle = WeaponIdleInputs {
        ads_idle_amount: w.f("fAdsIdleAmount"),
        hip_idle_amount: w.f("fHipIdleAmount"),
        ads_idle_speed: w.f("adsIdleSpeed"),
        hip_idle_speed: w.f("hipIdleSpeed"),
        idle_crouch_factor: w.f("fIdleCrouchFactor"),
        idle_prone_factor: w.f("fIdleProneFactor"),
    };
    f.penetrate_type = w.i("penetrateType");
    f.rifle_bullet = w.b("bRifleBullet");
    f.max_ammo = w.i("iMaxAmmo");
    f.start_ammo = w.i("iStartAmmo");
    f.damage = w.i("damage");
    f.min_damage = w.i("minDamage");
    f.min_player_damage = w.i("minPlayerDamage");
    f.max_damage_range = w.f("fMaxDamageRange");
    f.min_damage_range = w.f("fMinDamageRange");
    f.kill_icon_ratio = w.i("killIconRatio");
    f.flip_kill_icon = w.b("flipKillIcon");
    f.reload_ammo_add = w.i("iReloadAmmoAdd");
    f.reload_start_add = w.i("iReloadStartAdd");
    f.no_partial_reload = w.b("bNoPartialReload");
    f.bolt_action = w.b("bBoltAction");
    f.segmented_reload = w.b("bSegmentedReload");
    f.auto_aim_range = w.f("autoAimRange");
    f.aim_assist_range = w.f("aimAssistRange");
    f.aim_assist_range_ads = w.f("aimAssistRangeAds");
    f.cook_off_hold = w.b("bCookOffHold");
    f.clip_only = w.b("bClipOnly");
    f.has_detonator = w.b("hasDetonator");
    f.projectile_rotates = w.b("rotate");
    f.timed_detonation = w.b("timedDetonation");
    f.proj_impact_explode = w.b("bProjImpactExplode");
    f.explosion_radius = w.i("iExplosionRadius");
    f.explosion_radius_min = w.i("iExplosionRadiusMin");
    f.explosion_inner_damage = w.i("iExplosionInnerDamage");
    f.explosion_outer_damage = w.i("iExplosionOuterDamage");
    f.damage_cone_angle = w.f("damageConeAngle");
    f.stickiness = w.i("stickiness");
    f.projectile_speed = w.i("iProjectileSpeed");
    f.projectile_speed_up = w.i("iProjectileSpeedUp");
    f.projectile_speed_forward = w.i("iProjectileSpeedForward");
    f.projectile_activate_dist = w.i("iProjectileActivateDist");
    f.require_lock_to_fire = w.b("requireLockonToFire");
    // (19 hit locations, IW4's order; the rest take 1.)
    f.location_damage_mult = Some(std::array::from_fn(|k| {
        if k < 19 { w.fk("locationDamageMultipliers", k) } else { 1.0 }
    }));
    // (No dual wield in either game.)
    f.no_dual_wield = true;
    f.kick.gun_max_pitch = w.f("fGunMaxPitch");
    f.kick.gun_max_yaw = w.f("fGunMaxYaw");
    f.kick.ads_gun_kick_reduced_kick_bullets = w.i("adsGunKickReducedKickBullets");
    f.kick.ads_gun_kick_reduced_kick_percent = w.f("adsGunKickReducedKickPercent");
    f.kick.ads_gun_kick_pitch_min = w.f("fAdsGunKickPitchMin");
    f.kick.ads_gun_kick_pitch_max = w.f("fAdsGunKickPitchMax");
    f.kick.ads_gun_kick_yaw_min = w.f("fAdsGunKickYawMin");
    f.kick.ads_gun_kick_yaw_max = w.f("fAdsGunKickYawMax");
    f.kick.ads_gun_kick_accel = w.f("fAdsGunKickAccel");
    f.kick.ads_gun_kick_speed_max = w.f("fAdsGunKickSpeedMax");
    f.kick.ads_gun_kick_speed_decay = w.f("fAdsGunKickSpeedDecay");
    f.kick.ads_gun_kick_static_decay = w.f("fAdsGunKickStaticDecay");
    f.kick.ads_view_kick_pitch_min = w.f("fAdsViewKickPitchMin");
    f.kick.ads_view_kick_pitch_max = w.f("fAdsViewKickPitchMax");
    f.kick.ads_view_kick_yaw_min = w.f("fAdsViewKickYawMin");
    f.kick.ads_view_kick_yaw_max = w.f("fAdsViewKickYawMax");
    f.kick.hip_gun_kick_reduced_kick_bullets = w.i("hipGunKickReducedKickBullets");
    f.kick.hip_gun_kick_reduced_kick_percent = w.f("hipGunKickReducedKickPercent");
    f.kick.hip_gun_kick_pitch_min = w.f("fHipGunKickPitchMin");
    f.kick.hip_gun_kick_pitch_max = w.f("fHipGunKickPitchMax");
    f.kick.hip_gun_kick_yaw_min = w.f("fHipGunKickYawMin");
    f.kick.hip_gun_kick_yaw_max = w.f("fHipGunKickYawMax");
    f.kick.hip_gun_kick_accel = w.f("fHipGunKickAccel");
    f.kick.hip_gun_kick_speed_max = w.f("fHipGunKickSpeedMax");
    f.kick.hip_gun_kick_speed_decay = w.f("fHipGunKickSpeedDecay");
    f.kick.hip_gun_kick_static_decay = w.f("fHipGunKickStaticDecay");
    f.kick.hip_view_kick_pitch_min = w.f("fHipViewKickPitchMin");
    f.kick.hip_view_kick_pitch_max = w.f("fHipViewKickPitchMax");
    f.kick.hip_view_kick_yaw_min = w.f("fHipViewKickYawMin");
    f.kick.hip_view_kick_yaw_max = w.f("fHipViewKickYawMax");
    f.sway = WeaponSwayFacts {
        sway_max_angle: w.f("swayMaxAngle"),
        sway_lerp_speed: w.f("swayLerpSpeed"),
        sway_pitch_scale: w.f("swayPitchScale"),
        sway_yaw_scale: w.f("swayYawScale"),
        sway_horiz_scale: w.f("swayHorizScale"),
        sway_vert_scale: w.f("swayVertScale"),
        sway_shell_shock_scale: w.f("swayShellShockScale"),
        ads_sway_max_angle: w.f("adsSwayMaxAngle"),
        ads_sway_lerp_speed: w.f("adsSwayLerpSpeed"),
        ads_sway_pitch_scale: w.f("adsSwayPitchScale"),
        ads_sway_yaw_scale: w.f("adsSwayYawScale"),
        ads_sway_horiz_scale: w.f("adsSwayHorizScale"),
        ads_sway_vert_scale: w.f("adsSwayVertScale"),
    };
    f
}

fn legacy_sounds(w: &LegacyWeaponView) -> WeaponSoundAliases {
    let s = |m: &str| w.sound(m);
    WeaponSoundAliases {
        notetrack_convention: NotetrackConvention::SoundMap,
        fire: s("fireSound"),
        fire_player: s("fireSoundPlayer"),
        empty_fire: s("emptyFireSound"),
        empty_fire_player: s("emptyFireSoundPlayer"),
        melee_swipe: s("meleeSwipeSound"),
        melee_swipe_player: s("meleeSwipeSoundPlayer"),
        melee_hit: s("meleeHitSound"),
        melee_miss: s("meleeMissSound"),
        pickup: s("pickupSound"),
        pickup_player: s("pickupSoundPlayer"),
        ammo_pickup: s("ammoPickupSound"),
        ammo_pickup_player: s("ammoPickupSoundPlayer"),
        pullback: s("pullbackSound"),
        pullback_player: s("pullbackSoundPlayer"),
        reload: s("reloadSound"),
        reload_player: s("reloadSoundPlayer"),
        reload_empty: s("reloadEmptySound"),
        reload_empty_player: s("reloadEmptySoundPlayer"),
        reload_start: s("reloadStartSound"),
        reload_start_player: s("reloadStartSoundPlayer"),
        reload_end: s("reloadEndSound"),
        reload_end_player: s("reloadEndSoundPlayer"),
        rechamber: s("rechamberSound"),
        rechamber_player: s("rechamberSoundPlayer"),
        alt_switch: s("altSwitchSound"),
        alt_switch_player: s("altSwitchSoundPlayer"),
        raise: s("raiseSound"),
        raise_player: s("raiseSoundPlayer"),
        first_raise: s("firstRaiseSound"),
        first_raise_player: s("firstRaiseSoundPlayer"),
        putaway: s("putawaySound"),
        putaway_player: s("putawaySoundPlayer"),
        proj_explosion: s("projExplosionSound"),
        projectile: s("projectileSound"),
        proj_ignition_sound: s("projIgnitionSound"),
        fire_loop: s("fireLoopSound"),
        fire_loop_player: s("fireLoopSoundPlayer"),
        fire_stop: s("fireStopSound"),
        fire_stop_player: s("fireStopSoundPlayer"),
        fire_last: s("fireLastSound"),
        fire_last_player: s("fireLastSoundPlayer"),
        notetrack_sound_map: w.notetrack_sounds.clone(),
        bounce: std::array::from_fn(|surf| w.bounce.get(surf).filter(|n| !n.is_empty()).cloned()),
        ..WeaponSoundAliases::default()
    }
}

impl WeaponCatalog {
    /// bo2mp: one CoD4 or World at War weapon, named without its `_mp`
    /// (`ak47_mp` is `ak47`; a Black Ops II match gives it as
    /// `cod4_ak47_mp`, World at War's as `waw_<gun>_mp`). False for a
    /// weapon without a name.
    pub fn capture_legacy(&mut self, w: &LegacyWeaponView) -> bool {
        let name = w.catalog_name();
        if name.is_empty() {
            return false;
        }
        let opt = |s: &str| (!s.is_empty()).then(|| s.to_owned());
        let combat_fx = WeaponCombatFx {
            view_flash_hint: w.asset("viewFlashEffect"),
            world_flash_hint: w.asset("worldFlashEffect"),
            view_shell_eject_hint: w.asset("viewShellEjectEffect"),
            world_shell_eject_hint: w.asset("worldShellEjectEffect"),
            view_last_shot_eject_hint: w.asset("viewLastShotEjectEffect"),
            world_last_shot_eject_hint: w.asset("worldLastShotEjectEffect"),
            explosion_hint: w.asset("projExplosionEffect"),
            ..WeaponCombatFx::default()
        };
        let reticle = WeaponReticleAssets {
            center_material: w.asset("reticleCenter"),
            side_material: w.asset("reticleSide"),
            center_authored: w.asset("reticleCenter").is_some(),
            side_authored: w.asset("reticleSide").is_some(),
            center_size: w.i("iReticleCenterSize"),
            side_size: w.i("iReticleSideSize"),
            ..WeaponReticleAssets::default()
        };
        let alternate = opt(&w.alt_weapon_name).map(|a| {
            let a = a.to_ascii_lowercase();
            a.strip_suffix("_mp").unwrap_or(&a).to_owned()
        });
        self.entries.push(CatalogWeapon {
            namespace: self.capture_ns,
            name,
            alternate_weapon: alternate,
            weap_def: None,
            display_name_key: opt(&w.display_name),
            reticle,
            hud_material_edges: WeaponHudMaterialEdges::default(),
            overlay_material: w.asset("overlayMaterial").or_else(|| w.asset("overlayMaterialLowRes")),
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
            hud_icon: w.asset("hudIcon"),
            hud_icon_slot: None,
            pickup_icon: None,
            pickup_icon_slot: None,
            pickup_icon_image: None,
            pickup_icon_ratio: 0,
            hud_icon_ratio: w.i("hudIconRatio"),
            hud_icon_image: None,
            dpad_icon: w.asset("dpadIcon"),
            dpad_icon_image: None,
            dpad_icon_atlas: None,
            dpad_icon_ratio: w.i("dpadIconRatio"),
            kill_icon: w.asset("killIcon"),
            kill_icon_slot: None,
            kill_icon_image: None,
            proj_trail: w.asset("projTrailEffect"),
            proj_trail_slot: None,
            proj_beacon: None,
            proj_beacon_slot: None,
            proj_ignition: w.asset("projIgnitionEffect"),
            proj_ignition_slot: None,
            projectile_fx: WeaponProjectileFx::default(),
            gun_xmodel: w.gun_model.clone(),
            // BO2 draws the player character's arms with every weapon.
            hand_xmodel: None,
            world_model: w.world_model.clone(),
            projectile_model: w.asset("projectileModel"),
            rocket_model: w.asset("rocketModel"),
            knife_xmodel: w.asset("knifeModel"),
            sz_xanims: remap_legacy_sz_xanims(w.game, &w.xanims),
            sz_xanims_right: [const { None }; WEAPON_ANIM_SLOTS],
            sz_xanims_left: [const { None }; WEAPON_ANIM_SLOTS],
            dual_wield_weapon: None,
            hide_tags: w.hide_tags.clone(),
            ads_view_attachments: Vec::new(),
            view_attachments: Vec::new(),
            sounds: legacy_sounds(w),
            combat_fx,
            combat_slots: CombatFxSlots::default(),
            facts: legacy_body_facts(w),
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anim_slots_line_up() {
        let mut src = vec![String::new(); 35];
        src[1] = "Idle".into();
        src[31] = "ads_up_iw3".into();
        src[33] = "ads_up_t4".into();
        let iw3 = remap_legacy_sz_xanims(LegacyGame::Iw3, &src[..33]);
        assert_eq!(iw3[weap_anim::IDLE].as_deref(), Some("idle"));
        assert_eq!(iw3[weap_anim::ADS_UP].as_deref(), Some("ads_up_iw3"));
        let t4 = remap_legacy_sz_xanims(LegacyGame::T4, &src);
        assert_eq!(t4[weap_anim::ADS_UP].as_deref(), Some("ads_up_t4"));
    }

    #[test]
    fn classes_follow_iw4() {
        // CoD4 smg (2) is IW4's smg (3); pistol 4 -> 5; rocket launcher 6 -> 7.
        assert_eq!(remap_legacy_weap_class(LegacyGame::Iw3, 2), 3);
        assert_eq!(remap_legacy_weap_class(LegacyGame::Iw3, 4), 5);
        assert_eq!(remap_legacy_weap_class(LegacyGame::T4, 6), 7);
        assert_eq!(remap_legacy_impact_type(LegacyGame::Iw3, 4), 5);
    }
}
