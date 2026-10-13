use weapon_iw4::WeaponCombatFacts;

pub fn bullet_damage_at_distance(facts: &WeaponCombatFacts, dist: f32) -> i32 {
    if let Some(d) = damage_over_points(&facts.damage_points, dist) {
        return d.max(0);
    }
    let base = facts.damage;
    let min = if facts.min_damage > 0 {
        facts.min_damage
    } else {
        base
    };
    let mut damage = base;
    let range = facts.min_damage_range - facts.max_damage_range;
    if base != min && range != 0.0 {
        if facts.max_damage_range <= dist {
            if facts.min_damage_range <= dist {
                damage = min;
            } else {
                let lerp = (dist - facts.max_damage_range) / range;
                let lerp = lerp.clamp(0.0, 1.0);
                damage = ((base as f32) * (1.0 - lerp) + (min as f32) * lerp) as i32;
            }
        }
    }
    damage.max(0)
}

/// Black Ops II's damage at a distance from its (range, damage) points
/// (points with no range are unused): the first point's damage up to its
/// range, straight between neighbouring points, the last's beyond. None
/// when the gun has no points (the other games' guns).
pub fn damage_over_points(points: &[[f32; 2]; 6], dist: f32) -> Option<i32> {
    let mut used = points.iter().filter(|p| p[0] > 0.0);
    let first = used.next()?;
    if dist <= first[0] {
        return Some(first[1] as i32);
    }
    let mut prev = *first;
    for p in used {
        if dist <= p[0] {
            let span = p[0] - prev[0];
            let t = if span > 0.0 { (dist - prev[0]) / span } else { 1.0 };
            return Some((prev[1] + (p[1] - prev[1]) * t) as i32);
        }
        prev = *p;
    }
    Some(prev[1] as i32)
}

pub fn angles_to_forward(angles: [f32; 3]) -> [f32; 3] {
    let yaw = angles[1].to_radians();
    let pitch = angles[0].to_radians();
    let cy = yaw.cos();
    let sy = yaw.sin();
    let sp = pitch.sin();
    let cp = pitch.cos();
    [cp * cy, cp * sy, -sp]
}

/// bo2mp: a bolt-action sniper rifle kills with any hit (his rule: every
/// sniper is one shot, except those that fire in quick succession: the SVU,
/// XPR-50, Barrett, WA2000, Dragunov ...). The damage a hit does then,
/// before the wall it went through takes its share.
pub const ONE_SHOT_SNIPER_DAMAGE: f32 = 1000.0;

/// Sniper rifles by their base names (Black Ops II files them as rifles,
/// so the class alone does not tell): every game's in the catalogue.
const SNIPER_BASES: &[&str] = &[
    "dsr50", "ballista", "svu", "as50", "cheytac", "barrett", "wa2000", "m21", "l96a1", "msr", "rsass",
    "dragunov", "psg1", "l118a", "m40a3", "m40", "r700", "remington700", "mosin", "kar98k", "springfield",
    "type99rifle", "ptrs41", "usr", "vks", "intervention", "m82", "aw50", "svd", "lynx", "l115",
];

/// A sniper rifle: one of the names above, or a gun that looks down a scope
/// overlay (any sniper sight), not a pistol, shotgun or launcher.
pub fn is_sniper(facts: &WeaponCombatFacts, script_name: &str) -> bool {
    let name = script_name.rsplit(':').next().unwrap_or(script_name).to_ascii_lowercase();
    let base = name.trim_start_matches("iw5_").trim_start_matches("t5_");
    let named = SNIPER_BASES.iter().any(|b| {
        base.strip_prefix(b).is_some_and(|rest| rest.is_empty() || rest.starts_with('_') || rest.starts_with('+') || rest.starts_with("mp"))
    });
    // IW4 numbering after the remap: 0 rifle, 1 sniper.
    named || facts.weap_class == 1 || (facts.overlay_reticle != 0 && facts.weap_class == 0)
}

/// One hit kills: a sniper that does not fire in quick succession (bolt
/// action, or at least half a second between shots).
pub fn one_shot_sniper(facts: &WeaponCombatFacts, script_name: &str) -> bool {
    is_sniper(facts, script_name) && (facts.bolt_action || facts.fire_time_ms >= 500)
}

#[cfg(test)]
mod tests {
    use super::damage_over_points;
    use super::{one_shot_sniper, is_sniper};
    use weapon_iw4::WeaponCombatFacts;

    fn gun(bolt: bool, fire_ms: i32, overlay: i32) -> WeaponCombatFacts {
        let mut f = WeaponCombatFacts::none();
        f.bolt_action = bolt;
        f.fire_time_ms = fire_ms;
        f.overlay_reticle = overlay;
        f
    }

    #[test]
    fn bolt_snipers_kill_in_one_rapid_ones_do_not() {
        assert!(one_shot_sniper(&gun(true, 1100, 1), "dsr50_mp"));
        assert!(one_shot_sniper(&gun(true, 900, 0), "ballista_mp+acog"));
        assert!(one_shot_sniper(&gun(true, 1200, 1), "iw4:cheytac_mp"));
        assert!(!one_shot_sniper(&gun(false, 250, 1), "svu_mp"));
        assert!(!one_shot_sniper(&gun(false, 200, 1), "as50_mp"));
        assert!(!one_shot_sniper(&gun(false, 150, 1), "iw4:barrett_mp"));
        assert!(!one_shot_sniper(&gun(true, 1000, 0), "remington870_mp"));
        assert!(!is_sniper(&gun(false, 100, 0), "an94_mp"));
    }

    /// The MP7's points as Black Ops II's common_mp has them.
    const MP7: [[f32; 2]; 6] =
        [[800.0, 33.0], [801.0, 22.0], [1100.0, 22.0], [0.0, 0.0], [0.0, 0.0], [1100.0, 18.0]];
    /// The SWAT-556's (burst): five tiers.
    const SARITCH: [[f32; 2]; 6] =
        [[1000.0, 50.0], [1001.0, 50.0], [1500.0, 50.0], [1501.0, 49.0], [3000.0, 49.0], [3001.0, 40.0]];

    #[test]
    fn black_ops_ii_damage_steps_between_its_points() {
        assert_eq!(damage_over_points(&MP7, 10.0), Some(33));
        assert_eq!(damage_over_points(&MP7, 800.0), Some(33));
        assert_eq!(damage_over_points(&MP7, 900.0), Some(22));
        assert_eq!(damage_over_points(&MP7, 1099.0), Some(22));
        assert_eq!(damage_over_points(&MP7, 5000.0), Some(18));
        assert_eq!(damage_over_points(&SARITCH, 1400.0), Some(50));
        assert_eq!(damage_over_points(&SARITCH, 2000.0), Some(49));
        assert_eq!(damage_over_points(&SARITCH, 4000.0), Some(40));
        assert_eq!(damage_over_points(&[[0.0; 2]; 6], 100.0), None);
    }
}
