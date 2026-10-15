use asset_game::{WeaponBodyFacts, WeaponRegistry};
use weapon_iw4::{
    CapturedCombatInput, HITLOC_COUNT, LOCATION_DAMAGE_IDENTITY, MissingCombatFacts,
    WeaponCombatFacts, bake_location_damage, location_damage_is_valid,
};

pub(crate) fn validated_facts(
    f: WeaponBodyFacts,
    melee_charge_anim: bool,
    global_location: Option<[f32; HITLOC_COUNT]>,
) -> Result<WeaponCombatFacts, MissingCombatFacts> {
    let fire_type = weapon_iw4::FireType::from_i32(f.fire_type)
        .map_err(|_| MissingCombatFacts::UnknownFireType)?;

    let segmented_reload = f.segmented_reload && f.reload_start_time_ms > 0;
    let reload_ammo_add = if f.reload_ammo_add > 0 {
        f.reload_ammo_add
    } else if segmented_reload {
        1
    } else {
        0
    };
    if let Some(table) = f.location_damage_mult {
        if !location_damage_is_valid(&table) {
            return Err(MissingCombatFacts::LocationDamage);
        }
    }
    WeaponCombatFacts::try_from_captured(CapturedCombatInput {
        fire_time_ms: f.fire_time_ms,
        intro_fire_time_ms: f.intro_fire_time_ms,
        intro_fire_length: f.intro_fire_length,
        fire_delay_ms: f.fire_delay_ms,
        raise_time_ms: f.raise_time_ms,
        drop_time_ms: f.drop_time_ms,
        first_raise_time_ms: f.first_raise_time_ms,
        empty_raise_time_ms: f.empty_raise_time_ms,
        empty_drop_time_ms: f.empty_drop_time_ms,
        last_fire_time_ms: f.last_fire_time_ms,
        alternate_weapon: 0,
        alternate_raise_time_ms: f.alternate_raise_time_ms,
        alternate_drop_time_ms: f.alternate_drop_time_ms,
        reload_time_ms: f.reload_time_ms,
        reload_empty_time_ms: f.reload_empty_time_ms,
        clip_size: f.clip_size,
        start_ammo: f.start_ammo_rounds(),
        max_ammo: f.max_ammo_rounds(),
        ammo_index: f.ammo_index,
        clip_index: f.clip_index,
        fire_type: f.fire_type,
        weap_type: f.weap_type,
        weap_class: f.weap_class,
        player_anim_type: f.player_anim_type,
        inventory_type: f.inventory_type,

        impact_type: f.impact_type,
        shots_per_fire: f.shots_per_fire,

        // (Black Ops II's own `iBurstDelayTime` when the gun has one.)
        burst_cooldown_ms: if fire_type.is_burst() {
            if f.burst_delay_ms > 0 { f.burst_delay_ms } else { 200 }
        } else {
            0
        },

        bolt_action: f.bolt_action,
        shots_before_rechamber: f.shots_before_rechamber,
        rechamber_time_ms: f.rechamber_time_ms,
        rechamber_bolt_time_ms: f.rechamber_bolt_time_ms,
        rechamber_bolt_delay_ms: f.rechamber_bolt_delay_ms,
        segmented_reload,
        reload_start_time_ms: f.reload_start_time_ms,
        reload_end_time_ms: f.reload_end_time_ms,
        reload_ammo_add,
        reload_add_time_ms: f.reload_add_time_ms,
        reload_empty_add_time_ms: f.reload_empty_add_time_ms,
        reload_start_add_time_ms: f.reload_start_add_time_ms,
        reload_start_add: f.reload_start_add,
        no_partial_reload: f.no_partial_reload,
        unlimited_ammo: f.unlimited_ammo,
        dual_mag: f.dual_mag,
        spread_before_fire_add: f.spread_before_fire_add,
        inherits_perks: f.inherits_perks,
        sprint_raise_time_ms: f.sprint_raise_time_ms,
        sprint_drop_time_ms: f.sprint_drop_time_ms,
        damage_points: f.damage_points,
        damage: f.damage,
        min_damage: f.min_damage,
        max_damage_range: f.max_damage_range,
        min_damage_range: f.min_damage_range,
        hip_spread_stand_min: f.hip_spread_stand_min,
        hip_spread_ducked_min: f.hip_spread_ducked_min,
        hip_spread_prone_min: f.hip_spread_prone_min,
        hip_spread_stand_max: f.hip_spread_stand_max,
        hip_spread_ducked_max: f.hip_spread_ducked_max,
        hip_spread_prone_max: f.hip_spread_prone_max,
        hip_spread_decay_rate: f.hip_spread_decay_rate,
        hip_spread_fire_add: f.hip_spread_fire_add,
        hip_spread_turn_add: f.hip_spread_turn_add,
        hip_spread_move_add: f.hip_spread_move_add,
        hip_spread_ducked_decay: f.hip_spread_ducked_decay,
        hip_spread_prone_decay: f.hip_spread_prone_decay,
        ads_spread: f.ads_spread,
        aim_down_sight: f.aim_down_sight,
        no_ads_when_mag_empty: f.no_ads_when_mag_empty,
        ads_in_rate: f.ads_in_rate,
        ads_out_rate: f.ads_out_rate,
        rechamber_while_ads: f.rechamber_while_ads,
        reload_lowers_ads: f.reload_lowers_ads,
        position_reload_trans_ms: f.position_reload_trans_ms,
        ads_fire_only: f.ads_fire_only,
        optic_switch: f.optic_switch,
        melee_damage: f.melee_damage,
        overlay_reticle: f.overlay_reticle,
        melee_time_ms: f.melee_time_ms,
        melee_delay_ms: f.melee_delay_ms,
        melee_charge_time_ms: f.melee_charge_time_ms,
        melee_charge_delay_ms: f.melee_charge_delay_ms,
        melee_charge_anim,
        knife_model: f.knife_model,
        quick_raise_time_ms: f.quick_raise_time_ms,
        quick_drop_time_ms: f.quick_drop_time_ms,
        select_requires_ammo: f.select_requires_ammo,
        offhand_hold_is_cancelable: f.offhand_hold_is_cancelable,
        ads_gun_kick_reduced_kick_bullets: f.kick.ads_gun_kick_reduced_kick_bullets,
        hip_gun_kick_reduced_kick_bullets: f.kick.hip_gun_kick_reduced_kick_bullets,
        location_damage: bake_location_damage(
            f.weap_type,
            f.weap_class,
            global_location.unwrap_or(LOCATION_DAMAGE_IDENTITY),
            f.location_damage_mult,
        ),
    })
}

pub fn from_registry(
    weapons: &WeaponRegistry,
    global_location: Option<[f32; HITLOC_COUNT]>,
) -> Vec<WeaponCombatFacts> {
    let mut refused = Vec::new();
    let rows = (0..=weapons.len())
        .map(|i| {
            if i == 0 {
                return WeaponCombatFacts::none();
            }
            if !weapons.configuration_supported(i as u32) {
                diag::warn!(
                    Sim,
                    "unsupported T5 dual-hand animation configuration: {}",
                    weapons.name_of(i as u32)
                );
                return WeaponCombatFacts::none();
            }
            let Some(f) = weapons.facts_of(i as u32) else {
                return WeaponCombatFacts::none();
            };
            let charge_anim_of = |w: u32| {
                weapons
                    .sz_xanims_of(w)
                    .and_then(|t| t.get(8))
                    .and_then(|s| s.as_ref())
                    .is_some_and(|s| !s.is_empty())
            };
            let mut charge_anim = charge_anim_of(i as u32);
            let mut f = f;
            let melee_weapon = weapons.melee_weapon_of(i as u32);
            if melee_weapon != i as u32
                && let Some(m) = weapons.facts_of(melee_weapon)
            {
                f.melee_damage = m.melee_damage;
                f.melee_time_ms = m.melee_time_ms;
                f.melee_delay_ms = m.melee_delay_ms;
                f.melee_charge_time_ms = m.melee_charge_time_ms;
                f.melee_charge_delay_ms = m.melee_charge_delay_ms;
                charge_anim = charge_anim_of(melee_weapon);
                // Nonzero sends the gun through MELEE_END's quick raise after the knife.
                f.knife_model = melee_weapon;
            }
            let mut facts =
                validated_facts(f, charge_anim, global_location).unwrap_or_else(|reason| {
                    refused.push(format!("{}({reason:?})", weapons.name_of(i as u32)));
                    WeaponCombatFacts::none()
                });
            facts.sprint_cycle_scale = f.bob.sprint_cycle_scale;
            facts.ducked_sprint_cycle_scale = f.bob.ducked_sprint_cycle_scale;
            facts.dtp_cycle_scale = f.bob.dtp_cycle_scale;
            facts.alternate_weapon = weapons.alternate_of(i as u32);
            facts.dual_wield_weapon = weapons.dual_wield_weapon_of(i as u32);
            facts.spawned_grenade = weapons.spawned_grenade_of(i as u32);
            facts.retrievable = f.retrievable;
            facts.aim_assist = weapon_iw4::AimAssistRanges {
                auto_aim: f.auto_aim_range,
                hip: f.aim_assist_range,
                ads: f.aim_assist_range_ads,
            };
            facts
        })
        .collect();
    if !refused.is_empty() {
        diag::info!(
            Sim,
            "combat facts refused for {} weapons: {}",
            refused.len(),
            refused.join(" ")
        );
    }
    rows
}

/// bo2mp: under Black Ops II's rules every gun's `playerAnimType` is BO2's
/// (its place in BO2's playeranimtypes.txt, as BO2's script reads it): a
/// gun from a Modern Warfare-numbered game (MW2's riot shield is its 15,
/// BO2's "rearclip") takes BO2's type of the same name.
pub fn bo2_player_anim_types(
    combat: &mut [WeaponCombatFacts],
    weapons: &WeaponRegistry,
    script: &sim::t6_playeranim::T6PlayerAnims,
) {
    for (i, facts) in combat.iter_mut().enumerate().skip(1) {
        if facts.is_usable() && weapons.player_anim_type_iw_numbered(i as u32) {
            facts.player_anim_type = script.guest_anim_type(facts.player_anim_type);
        }
    }
}

/// bo2mp: the most rounds a guest gun carries on a Zombies map, past its
/// magazine (Black Ops II's RPD: 400).
const ZM_GUEST_MAX_RESERVE: i32 = 400;

/// bo2mp: other games' guns on a Zombies map (the Mystery Box's custom
/// guns) carry Zombies' proportions instead of multiplayer's. Black Ops
/// II's Zombies guns are its multiplayer guns retuned: more damage a
/// bullet and more magazines in reserve. Each guest gun takes the same
/// retuning, measured on this map: the median, over the guns the map's
/// own Zombies weapons share a family with (its `ak74u_zm` and Black Ops'
/// `bo1_ak74u_mp`), of Zombies' damage over the guest's (bullet guns), and
/// of the Zombies gun's reserve in magazines. With no such pair the gun's
/// damage stays and its reserve is 8 magazines. A gun arrives full (its
/// start ammo is its reserve), as a Zombies gun from the box does. Returns
/// the log line.
pub fn zombies_guest_proportions(combat: &mut [WeaponCombatFacts], weapons: &WeaponRegistry) -> String {
    use weapon_iw4::WEAPTYPE_BULLET;
    let guest = |i: u32| weapons.namespace_of(i).is_some_and(|ns| ns != asset_core::AssetNamespace::T6);
    let zm: std::collections::HashMap<String, usize> = (1..combat.len() as u32)
        .filter(|&i| weapons.namespace_of(i) == Some(asset_core::AssetNamespace::T6))
        .filter_map(|i| {
            let name = weapons.script_name_of(i).to_ascii_lowercase();
            let stem = name.strip_suffix("_mp").unwrap_or(&name);
            (stem.ends_with("_zm") && !stem.contains("upgraded") && !stem.contains('+'))
                .then(|| (bo2_profile::guest::family_of_zm_weapon(stem), i as usize))
        })
        .collect();
    let (mut damage, mut mags) = (Vec::new(), Vec::new());
    let mut pairs = Vec::new();
    for i in 1..combat.len() as u32 {
        let name = weapons.script_name_of(i);
        if !guest(i) || name.contains('+') || !bo2_profile::guest::is_guest_name(&name) {
            continue;
        }
        let family = bo2_profile::guest::family_of_reference(name.strip_suffix("_mp").unwrap_or(&name));
        let Some(&z) = zm.get(&family) else { continue };
        let (g, z) = (&combat[i as usize], &combat[z]);
        if !g.is_usable() || !z.is_usable() || g.weap_type != WEAPTYPE_BULLET || z.weap_type != WEAPTYPE_BULLET {
            continue;
        }
        if g.damage > 0 && z.damage > 0 {
            damage.push(z.damage as f32 / g.damage as f32);
        }
        if z.clip_size > 0 && z.max_ammo > 0 {
            mags.push(z.max_ammo as f32 / z.clip_size as f32);
        }
        pairs.push(name);
    }
    let median = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied()
    };
    let damage_scale = median(&mut damage).map_or(1.0, |r| r.clamp(1.0, 6.0));
    let reserve_mags = median(&mut mags).map_or(8.0, |m| m.clamp(3.0, 15.0));
    let mut changed = 0;
    for i in 1..combat.len() as u32 {
        if !guest(i) || !combat[i as usize].is_usable() {
            continue;
        }
        // (A guest gun's upgrade from its own game's Pack-a-Punch is that
        // game's zombies weapon already: its stats stay its own.)
        if bo2_profile::guest::zm_upgraded(&weapons.script_name_of(i)).is_some_and(|u| !u.made) {
            continue;
        }
        let f = &mut combat[i as usize];
        if f.weap_type == WEAPTYPE_BULLET && f.damage > 0 && damage_scale != 1.0 {
            f.damage = (f.damage as f32 * damage_scale).round() as i32;
            f.min_damage = (f.min_damage as f32 * damage_scale).round() as i32;
            for point in &mut f.damage_points {
                point[1] *= damage_scale;
            }
        }
        if f.clip_size > 0 && !f.unlimited_ammo {
            let reserve = ((f.clip_size as f32 * reserve_mags).round() as i32).min(ZM_GUEST_MAX_RESERVE);
            f.max_ammo = f.max_ammo.max(reserve);
            f.start_ammo = f.max_ammo;
        }
        changed += 1;
    }
    format!(
        "bo2mp zombies guest guns: {changed} weapons at Zombies proportions: damage x{damage_scale:.2} ({} pairs), reserve {reserve_mags:.1} magazines ({} pairs, at most {ZM_GUEST_MAX_RESERVE}); measured on {}",
        damage.len(),
        mags.len(),
        pairs.join(" ")
    )
}

/// bo2mp: the most rounds a Black Ops II-style guest upgrade carries past
/// its magazine.
const ZM_GUEST_UPGRADE_MAX_RESERVE: i32 = 600;

/// bo2mp: the guest guns' Pack-a-Punch upgrades made Black Ops II-style
/// (`bo2_profile::guest::ZmUpgrade::made`, copies of the gun) are stronger
/// than the gun as the map's own upgrades are than theirs: the median, over
/// the map's Zombies guns with an upgrade (`ak74u_zm` and
/// `ak74u_upgraded_zm`), of the upgrade's damage over the gun's (bullet
/// guns), magazine over magazine, and reserve over reserve. With no such
/// pair: damage x1.8, magazine x1.5, reserve x1.6. Run after
/// `zombies_guest_proportions` (the copy starts as the gun at Zombies'
/// proportions). An upgrade from the gun's own game keeps that game's
/// stats. Returns the log line.
pub fn zombies_guest_upgrades(combat: &mut [WeaponCombatFacts], weapons: &WeaponRegistry) -> String {
    use weapon_iw4::WEAPTYPE_BULLET;
    let upgrades = bo2_profile::guest::zm_upgrades();
    if upgrades.is_empty() {
        return "bo2mp zombies guest upgrades: none".to_owned();
    }
    let t6: std::collections::HashMap<String, usize> = (1..combat.len() as u32)
        .filter(|&i| weapons.namespace_of(i) == Some(asset_core::AssetNamespace::T6))
        .filter_map(|i| {
            let name = weapons.script_name_of(i).to_ascii_lowercase();
            let stem = name.strip_suffix("_mp").unwrap_or(&name).to_owned();
            (stem.ends_with("_zm") && !stem.contains('+')).then_some((stem, i as usize))
        })
        .collect();
    let (mut damage, mut clip, mut reserve) = (Vec::new(), Vec::new(), Vec::new());
    let mut pairs = Vec::new();
    for (name, &i) in &t6 {
        let Some(stem) = name.strip_suffix("_zm").filter(|s| !s.contains("upgraded")) else { continue };
        let Some(&u) = t6.get(&format!("{stem}_upgraded_zm")) else { continue };
        let (g, up) = (&combat[i], &combat[u]);
        if !g.is_usable() || !up.is_usable() || g.unlimited_ammo || up.unlimited_ammo {
            continue;
        }
        if g.weap_type == WEAPTYPE_BULLET && up.weap_type == WEAPTYPE_BULLET && g.damage > 0 && up.damage > 0 {
            damage.push(up.damage as f32 / g.damage as f32);
        }
        if g.clip_size > 0 && up.clip_size > 0 {
            clip.push(up.clip_size as f32 / g.clip_size as f32);
        }
        if g.max_ammo > 0 && up.max_ammo > 0 {
            reserve.push(up.max_ammo as f32 / g.max_ammo as f32);
        }
        pairs.push(stem.to_owned());
    }
    pairs.sort();
    let median = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied()
    };
    let damage_scale = median(&mut damage).map_or(1.8, |r| r.clamp(1.0, 5.0));
    let clip_scale = median(&mut clip).map_or(1.5, |r| r.clamp(1.0, 3.0));
    let reserve_scale = median(&mut reserve).map_or(1.6, |r| r.clamp(1.0, 4.0));
    let mut made = 0;
    for u in upgrades.iter().filter(|u| u.made) {
        let index = |name: &str| weapons.resolve_index(name).ok().flatten().unwrap_or(0) as usize;
        let (b, i) = (index(&u.base), index(&u.upgraded));
        if b == 0 || i == 0 || i >= combat.len() || b >= combat.len() || !combat[b].is_usable() {
            continue;
        }
        let mut f = combat[b];
        // (Its own magazine and reserve, not the gun's.)
        f.ammo_index = 0;
        f.clip_index = 0;
        if f.weap_type == WEAPTYPE_BULLET && f.damage > 0 {
            f.damage = (f.damage as f32 * damage_scale).round() as i32;
            f.min_damage = (f.min_damage as f32 * damage_scale).round() as i32;
            for point in &mut f.damage_points {
                point[1] *= damage_scale;
            }
        }
        if f.clip_size > 0 && !f.unlimited_ammo {
            let clip = (f.clip_size as f32 * clip_scale).round() as i32;
            let reserve = ((f.max_ammo as f32 * reserve_scale).round() as i32).min(ZM_GUEST_UPGRADE_MAX_RESERVE.max(f.max_ammo));
            f.clip_size = clip.max(f.clip_size);
            f.max_ammo = reserve.max(f.max_ammo);
            f.start_ammo = f.max_ammo;
        }
        combat[i] = f;
        made += 1;
    }
    format!(
        "bo2mp zombies guest upgrades: {made} made Black Ops II-style, {} from their own games; damage x{damage_scale:.2} ({} pairs), magazine x{clip_scale:.2} ({} pairs), reserve x{reserve_scale:.2} ({} pairs); measured on {}",
        upgrades.iter().filter(|u| !u.made).count(),
        damage.len(),
        clip.len(),
        reserve.len(),
        pairs.join(" ")
    )
}

const T5_WEAPTYPE_MELEE: i32 = 7;
/// bo2zm: Black Ops II's `WEAPTYPE_MELEE` (its knives: `knife_zm`,
/// `bowie_knife_zm`, `tazer_knuckles_zm`), kept raw since `weap_type` is
/// remapped to IW4's gun types.
const T6_WEAPTYPE_MELEE: i32 = 7;

pub fn melee_only_from_registry(weapons: &WeaponRegistry) -> Vec<bool> {
    (0..=weapons.len())
        .map(|i| {
            weapons.facts_of(i as u32).is_some_and(|f| {
                (f.weap_type == T5_WEAPTYPE_MELEE || f.t6_weap_type == T6_WEAPTYPE_MELEE)
                    && f.fire_time_ms <= 0
                    && f.raise_time_ms <= 0
                    && f.clip_size <= 0
            })
        })
        .collect()
}

/// bo2zm: each weapon's own knife damage (BO2 hits with the melee weapon he
/// holds: the Bowie Knife cuts harder than the knife).
pub fn melee_damage_from_registry(weapons: &WeaponRegistry) -> Vec<i32> {
    (0..=weapons.len())
        .map(|i| weapons.facts_of(i as u32).map_or(0, |f| f.melee_damage))
        .collect()
}

pub fn script_sounds_from_registry(weapons: &WeaponRegistry) -> Vec<sim::WeaponScriptSounds> {
    (0..=weapons.len())
        .map(|i| {
            weapons
                .sounds_of(i as u32)
                .map(|s| sim::WeaponScriptSounds {
                    fire: s.fire.clone(),
                    fire_player: s.fire_player.clone(),
                    pickup: s.pickup.clone(),
                    pickup_player: s.pickup_player.clone(),
                    proj_explosion: s.proj_explosion.clone(),
                })
                .unwrap_or_default()
        })
        .collect()
}

pub fn pen_from_registry(weapons: &WeaponRegistry) -> Vec<weapon_iw4::BulletPenFacts> {
    (0..=weapons.len())
        .map(|i| {
            let Some(f) = weapons.facts_of(i as u32) else {
                return weapon_iw4::BulletPenFacts::default();
            };
            weapon_iw4::BulletPenFacts {
                penetrate_type: f.penetrate_type,
                penetrate_multiplier: f.penetrate_multiplier,
                rifle_bullet: f.rifle_bullet,
            }
        })
        .collect()
}

pub fn equipment_from_registry(weapons: &WeaponRegistry) -> Vec<sim::EquipmentRuntimeFacts> {
    (0..=weapons.len())
        .map(|index| {
            let Some(f) = weapons.facts_of(index as u32) else {
                return sim::EquipmentRuntimeFacts::default();
            };
            sim::EquipmentRuntimeFacts {
                offhand_class: f.offhand_class,
                start_ammo: f.start_ammo_rounds(),
                clip_size: f.clip_size,
                impact_damage: f.damage,
                fuse_time_ms: f.fuse_time_ms,
                hold_fire_time_ms: f.hold_fire_time_ms,
                cook_off_hold: f.cook_off_hold,
                has_detonator: f.has_detonator,
                detonate_delay_ms: f.detonate_delay_ms,
                detonate_time_ms: f.detonate_time_ms,
                projectile_rotates: f.projectile_rotates
                    || weapons.namespace_of(index as u32) != Some(asset_core::AssetNamespace::Iw4),
                stickiness: f.stickiness,
                timed_detonation: f.timed_detonation,
                proj_impact_explode: f.proj_impact_explode,
                bullet_impact_explode: f.bullet_impact_explode,
                stick_to_players: f.stick_to_players,
                ballistic_blade: weapons.namespace_of(index as u32)
                    == Some(asset_core::AssetNamespace::T5)
                    && weapons.name_of(index as u32) == "knife_ballistic_mp",
                explosion_radius: f.explosion_radius,
                explosion_radius_min: f.explosion_radius_min,
                explosion_inner_damage: f.explosion_inner_damage,
                explosion_outer_damage: f.explosion_outer_damage,
                damage_cone_angle: f.damage_cone_angle,
                missile_guidance: f.missile_guidance,
                ignition_delay_ms: f.ignition_delay_ms,
                require_lock_to_fire: f.require_lock_to_fire,
                projectile_speed: f.projectile_speed,
                projectile_speed_up: f.projectile_speed_up,
                projectile_speed_forward: f.projectile_speed_forward,
                projectile_speed_relative_up: f.projectile_speed_relative_up,
                projectile_parent_velocity: f.projectile_parent_velocity,
                rolling_radius: f.rolling_radius,
                projectile_activate_dist: f.projectile_activate_dist,
                projectile_explosion_type: f.projectile_explosion_type,
                weap_type: f.weap_type,
                weap_class: f.weap_class,
                parallel_bounce: f.parallel_bounce,
                perpendicular_bounce: f.perpendicular_bounce,
            }
        })
        .collect()
}

#[cfg(test)]
mod bo2_tests {
    use super::*;

    fn gun() -> WeaponBodyFacts {
        WeaponBodyFacts {
            fire_type: 2,
            fire_time_ms: 64,
            damage: 40,
            max_damage_range: 1000.0,
            min_damage_range: 2000.0,
            location_damage_mult: Some(LOCATION_DAMAGE_IDENTITY),
            ..WeaponBodyFacts::default()
        }
    }

    #[test]
    fn a_burst_waits_its_own_burst_delay() {
        let f = validated_facts(WeaponBodyFacts { burst_delay_ms: 112, ..gun() }, false, None).expect("facts");
        assert_eq!(f.burst_cooldown_ms, 112);
        let f = validated_facts(gun(), false, None).expect("facts");
        assert_eq!(f.burst_cooldown_ms, 200);
    }

    #[test]
    fn black_ops_ii_global_table_scales_a_limb_shot() {
        let table = asset_game::bo2_lochit_table();
        let baked = bake_location_damage(weapon_iw4::WEAPTYPE_BULLET, 0, table, Some(LOCATION_DAMAGE_IDENTITY));
        let slot = |n| weapon_iw4::hitloc_slot(n).unwrap();
        assert_eq!(baked[slot("left_leg_lower")], 0.5);
        assert_eq!(baked[slot("torso_mid")], 0.9);
    }
}
